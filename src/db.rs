use crate::error::ApiError;
use crate::config::DatabaseConfig;
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest};
use crate::models::post::{Post, CreatePostRequest};
use crate::models::vocabulary::{Vocabulary, CreateVocabularyRequest};
use deadpool_postgres::{Config, Pool, Runtime, Object};
use postgres_native_tls::MakeTlsConnector;
use native_tls::TlsConnector;
use tracing::{error, info, warn};

/// PostgreSQL への接続プールを握るリポジトリ層。
/// Deadpool の `Pool` を内部に保持し、各種ドメイン操作をメソッドとして提供する。
#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    /// 接続プールを構築し、起動時に疎通確認まで実施する。
    /// `async fn` なので `Database::new(config).await` のように `await` が必要。
    /// 
    /// # Arguments
    /// * `config` - The database configuration
    /// 
    /// # Returns
    /// * `Result<Self, ApiError>` - Database instance or error
    pub async fn new(config: DatabaseConfig) -> Result<Self, ApiError> {
        info!("Creating PostgreSQL connection pool for host: {}:{}", config.host, config.port);
        
        let pool = Self::create_pool(config).await?;
        
        // Test the connection pool
        let db = Database { pool };
        db.test_connection().await?;
        
        Ok(db)
    }

    /// Deadpool 用の `Config` を組み立ててプールを生成する内部関数。
    /// `match` で SSL モードを切り替え、`native_tls` で TLS コネクタを差し込んでいる点に注目。
    async fn create_pool(config: DatabaseConfig) -> Result<Pool, ApiError> {
        let mut pg_config = Config::new();
        
        // Set connection parameters
        pg_config.host = Some(config.host);
        pg_config.port = Some(config.port);
        pg_config.dbname = Some(config.database);
        pg_config.user = Some(config.username);
        pg_config.password = Some(config.password);
        
        // Configure SSL mode
        match config.ssl_mode.as_str() {
            "disable" => {
                pg_config.ssl_mode = Some(deadpool_postgres::SslMode::Disable);
            }
            "prefer" => {
                pg_config.ssl_mode = Some(deadpool_postgres::SslMode::Prefer);
            }
            "require" => {
                pg_config.ssl_mode = Some(deadpool_postgres::SslMode::Require);
            }
            _ => {
                warn!("Unknown SSL mode '{}', defaulting to 'require'", config.ssl_mode);
                pg_config.ssl_mode = Some(deadpool_postgres::SslMode::Require);
            }
        }
        
        // Configure connection pool
        pg_config.manager = Some(deadpool_postgres::ManagerConfig {
            recycling_method: deadpool_postgres::RecyclingMethod::Fast,
        });
        
        pg_config.pool = Some(deadpool_postgres::PoolConfig::new(config.max_connections as usize));
        
        // Create TLS connector for secure connections (required by Neon)
        let tls_connector = TlsConnector::builder()
            .build()
            .map_err(|e| {
                error!("Failed to create TLS connector: {}", e);
                ApiError::Database(format!("TLS connector creation failed: {}", e))
            })?;
        let tls = MakeTlsConnector::new(tls_connector);
        
        // Create the pool with TLS support
        pg_config.create_pool(Some(Runtime::Tokio1), tls)
            .map_err(|e| {
                error!("Failed to create connection pool: {}", e);
                ApiError::Database(format!("Connection pool creation failed: {}", e))
            })
    }

    /// プールから接続を借りる小さなラッパー。
    /// `deadpool_postgres::Pool::get` が返す `PoolError` を `ApiError` に変換する。
    async fn get_connection(&self) -> Result<Object, ApiError> {
        self.pool.get().await.map_err(ApiError::from)
    }

    /// `SELECT 1` を投げて DB が生きているか確認する。
    /// このようなシンプルなクエリは「ヘルスチェック」用としてよく使われる。
    pub async fn health_check(&self) -> Result<(), ApiError> {
        let client = self.get_connection().await?;
        
        client.execute("SELECT 1", &[])
            .await
            .map_err(|e| {
                error!("Database health check failed: {}", e);
                ApiError::Database(format!("Health check failed: {}", e))
            })?;
            
        info!("Database health check successful");
        Ok(())
    }

    /// アプリ起動時にテーブル群を CREATE する簡易マイグレーター。
    /// SQL をリテラル文字列で保持しておき、`client.execute` を順番に呼び出している。
    pub async fn migrate(&self) -> Result<(), ApiError> {
        info!("Running database migrations");
        
        let client = self.get_connection().await?;
        
        // Enable UUID extension if not already enabled
        let enable_uuid = "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"";
        client.execute(enable_uuid, &[])
            .await
            .map_err(|e| {
                error!("Failed to enable UUID extension: {}", e);
                ApiError::Database(format!("UUID extension error: {}", e))
            })?;
        
        // Create users table with PostgreSQL types
        let users_table = r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                name VARCHAR(255) NOT NULL,
                email VARCHAR(255) UNIQUE NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#;
        
        client.execute(users_table, &[])
            .await
            .map_err(|e| {
                error!("Failed to create users table: {}", e);
                ApiError::Database(format!("Users table creation failed: {}", e))
            })?;

        // Create index on email for users table
        let users_email_index = "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)";
        
        client.execute(users_email_index, &[])
            .await
            .map_err(|e| {
                error!("Failed to create users email index: {}", e);
                ApiError::Database(format!("Users email index creation failed: {}", e))
            })?;

        // Create posts table with PostgreSQL types and proper foreign key
        let posts_table = r#"
            CREATE TABLE IF NOT EXISTS posts (
                id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
                user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                title VARCHAR(500) NOT NULL,
                content TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#;
        
        client.execute(posts_table, &[])
            .await
            .map_err(|e| {
                error!("Failed to create posts table: {}", e);
                ApiError::Database(format!("Posts table creation failed: {}", e))
            })?;

        // Create indexes for posts table
        let posts_user_index = "CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id)";
        client.execute(posts_user_index, &[])
            .await
            .map_err(|e| {
                error!("Failed to create posts user_id index: {}", e);
                ApiError::Database(format!("Posts user_id index creation failed: {}", e))
            })?;

        let posts_created_index = "CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts(created_at DESC)";
        client.execute(posts_created_index, &[])
            .await
            .map_err(|e| {
                error!("Failed to create posts created_at index: {}", e);
                ApiError::Database(format!("Posts created_at index creation failed: {}", e))
            })?;

        // Create vocabulary table with SERIAL primary key
        let vocabulary_table = r#"
            CREATE TABLE IF NOT EXISTS vocabulary (
                id SERIAL PRIMARY KEY,
                en_word VARCHAR(200) NOT NULL,
                ja_word VARCHAR(200) NOT NULL,
                en_example TEXT,
                ja_example TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#;
        
        client.execute(vocabulary_table, &[])
            .await
            .map_err(|e| {
                error!("Failed to create vocabulary table: {}", e);
                ApiError::Database(format!("Vocabulary table creation failed: {}", e))
            })?;

        // Create index on en_word for vocabulary table
        let vocabulary_en_word_index = "CREATE INDEX IF NOT EXISTS idx_vocabulary_en_word ON vocabulary(en_word)";
        client.execute(vocabulary_en_word_index, &[])
            .await
            .map_err(|e| {
                error!("Failed to create vocabulary en_word index: {}", e);
                ApiError::Database(format!("Vocabulary en_word index creation failed: {}", e))
            })?;

        // Create index on ja_word for vocabulary table
        let vocabulary_ja_word_index = "CREATE INDEX IF NOT EXISTS idx_vocabulary_ja_word ON vocabulary(ja_word)";
        client.execute(vocabulary_ja_word_index, &[])
            .await
            .map_err(|e| {
                error!("Failed to create vocabulary ja_word index: {}", e);
                ApiError::Database(format!("Vocabulary ja_word index creation failed: {}", e))
            })?;

        // Create index on created_at for vocabulary table
        let vocabulary_created_index = "CREATE INDEX IF NOT EXISTS idx_vocabulary_created_at ON vocabulary(created_at DESC)";
        client.execute(vocabulary_created_index, &[])
            .await
            .map_err(|e| {
                error!("Failed to create vocabulary created_at index: {}", e);
                ApiError::Database(format!("Vocabulary created_at index creation failed: {}", e))
            })?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// `health_check` と似ているが、`Database::new` 直後にプール全体が機能するかの確認に使う。
    /// 失敗した場合は即座に `ApiError::Database` を返す。
    pub async fn test_connection(&self) -> Result<(), ApiError> {
        let client = self.get_connection().await?;
        
        // Simple query to test connection
        client.execute("SELECT 1", &[])
            .await
            .map_err(|e| {
                error!("Database connection test failed: {}", e);
                ApiError::Database(format!("Connection test failed: {}", e))
            })?;
            
        info!("Database connection test successful");
        Ok(())
    }

    // User repository operations

    /// ユーザー作成ロジック。
    /// `CreateUserRequest::validate` でビジネスルールを検証し、
    /// `request.into_user()` でドメインモデルに変換してから INSERT している。
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, ApiError> {
        // Validate the request
        request.validate().map_err(ApiError::Validation)?;
        
        let user = request.into_user();
        let client = self.get_connection().await?;
        
        let query = r#"
            INSERT INTO users (id, name, email, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, email, created_at, updated_at
        "#;
        
        let row = client.query_one(
            query,
            &[&user.id, &user.name, &user.email, &user.created_at, &user.updated_at]
        )
        .await
        .map_err(ApiError::from)?;
        
        let created_user = User {
            id: row.get(0),
            name: row.get(1),
            email: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
        };
        
        info!("Created user with id: {}", created_user.id);
        Ok(created_user)
    }

    /// UUID 文字列をパースし、単一行を取得する。
    /// `uuid::Uuid::parse_str` が失敗した場合は `ApiError::Validation` を返すのがポイント。
    pub async fn get_user_by_id(&self, user_id: &str) -> Result<User, ApiError> {
        // Parse the user_id string to UUID
        let uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|_| ApiError::Validation("Invalid user ID format".to_string()))?;
            
        let client = self.get_connection().await?;
        let query = "SELECT id, name, email, created_at, updated_at FROM users WHERE id = $1";
        
        let row = client.query_opt(query, &[&uuid])
            .await
            .map_err(ApiError::from)?;
        
        if let Some(row) = row {
            let user = User {
                id: row.get(0),
                name: row.get(1),
                email: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
            };
            
            Ok(user)
        } else {
            Err(ApiError::NotFound(format!("User with id {} not found", user_id)))
        }
    }

    /// 登録日時降順で全ユーザーを取得する。
    /// `rows.iter().map(|row| ...)` のクロージャ内で `tokio_postgres::Row` から型安全に取り出す。
    pub async fn get_all_users(&self) -> Result<Vec<User>, ApiError> {
        let client = self.get_connection().await?;
        let query = "SELECT id, name, email, created_at, updated_at FROM users ORDER BY created_at DESC";
        
        let rows = client.query(query, &[])
            .await
            .map_err(ApiError::from)?;
        
        let users: Vec<User> = rows.iter().map(|row| {
            User {
                id: row.get(0),
                name: row.get(1),
                email: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
            }
        }).collect();
        
        Ok(users)
    }

    /// 渡された `UpdateUserRequest` の Option 値に応じて動的に SQL を組み立てる。
    /// ベクタに `&(dyn ToSql + Sync)` を詰めるのは、Postgres のプレースホルダに順番対応させるため。
    pub async fn update_user(&self, user_id: &str, request: UpdateUserRequest) -> Result<User, ApiError> {
        // Validate the request
        request.validate().map_err(ApiError::Validation)?;
        
        // Parse the user_id string to UUID
        let uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|_| ApiError::Validation("Invalid user ID format".to_string()))?;
            
        let client = self.get_connection().await?;
        
        // Build dynamic query based on provided fields
        let mut query_parts = Vec::new();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
        let mut param_count = 1;
        
        // Always update the updated_at timestamp
        let updated_at = chrono::Utc::now();
        
        // Store normalized values to extend their lifetime
        let normalized_name = request.get_normalized_name();
        let normalized_email = request.get_normalized_email();
        
        if let Some(ref name) = normalized_name {
            query_parts.push(format!("name = ${}", param_count));
            params.push(name);
            param_count += 1;
        }
        
        if let Some(ref email) = normalized_email {
            query_parts.push(format!("email = ${}", param_count));
            params.push(email);
            param_count += 1;
        }
        
        // Add updated_at timestamp
        query_parts.push(format!("updated_at = ${}", param_count));
        params.push(&updated_at);
        param_count += 1;
        
        // Add WHERE clause parameter
        params.push(&uuid);
        
        let query = format!(
            "UPDATE users SET {} WHERE id = ${} RETURNING id, name, email, created_at, updated_at",
            query_parts.join(", "),
            param_count
        );
        
        let row = client.query_opt(&query, &params)
            .await
            .map_err(ApiError::from)?;
        
        if let Some(row) = row {
            let updated_user = User {
                id: row.get(0),
                name: row.get(1),
                email: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
            };
            
            info!("Updated user with id: {}", updated_user.id);
            Ok(updated_user)
        } else {
            Err(ApiError::NotFound(format!("User with id {} not found", user_id)))
        }
    }

    /// UUID をパースして DELETE を流すだけのシンプルな処理。
    /// テーブル定義側で `ON DELETE CASCADE` を付けているため、関連ポストも同時に消える。
    pub async fn delete_user(&self, user_id: &str) -> Result<(), ApiError> {
        // Parse the user_id string to UUID
        let uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|_| ApiError::Validation("Invalid user ID format".to_string()))?;
            
        let client = self.get_connection().await?;
        let query = "DELETE FROM users WHERE id = $1";
        
        let rows_affected = client.execute(query, &[&uuid])
            .await
            .map_err(ApiError::from)?;
        
        if rows_affected == 0 {
            Err(ApiError::NotFound(format!("User with id {} not found", user_id)))
        } else {
            info!("Deleted user with id: {} (cascade deleted {} posts)", user_id, rows_affected);
            Ok(())
        }
    }

    // Post repository operations
    // TODO: Post methods will be updated to use PostgreSQL syntax in task 4.4

    /// ポスト作成ロジック。
    /// 本文は `Option<String>` なので、NULL を許容する列への INSERT 例として読める。
    pub async fn create_post(&self, request: CreatePostRequest) -> Result<Post, ApiError> {
        // Validate the request
        request.validate().map_err(ApiError::Validation)?;
        
        let post = request.into_post();
        let client = self.get_connection().await?;
        
        let query = r#"
            INSERT INTO posts (id, user_id, title, content, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, title, content, created_at, updated_at
        "#;
        
        let row = client.query_one(
            query,
            &[&post.id, &post.user_id, &post.title, &post.content, &post.created_at, &post.updated_at]
        )
        .await
        .map_err(ApiError::from)?;
        
        let created_post = Post {
            id: row.get(0),
            user_id: row.get(1),
            title: row.get(2),
            content: row.get(3),
            created_at: row.get(4),
            updated_at: row.get(5),
        };
        
        info!("Created post with id: {}", created_post.id);
        Ok(created_post)
    }

    /// 単一ポストを UUID で検索する。
    /// `query_opt` を使うことで、存在しない場合に `Ok(None)` を返しつつ
    /// エラーと区別できる。
    pub async fn get_post_by_id(&self, post_id: &str) -> Result<Post, ApiError> {
        // Parse the post_id string to UUID
        let uuid = uuid::Uuid::parse_str(post_id)
            .map_err(|_| ApiError::Validation("Invalid post ID format".to_string()))?;
            
        let client = self.get_connection().await?;
        let query = "SELECT id, user_id, title, content, created_at, updated_at FROM posts WHERE id = $1";
        
        let row = client.query_opt(query, &[&uuid])
            .await
            .map_err(ApiError::from)?;
        
        if let Some(row) = row {
            let post = Post {
                id: row.get(0),
                user_id: row.get(1),
                title: row.get(2),
                content: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
            };
            
            Ok(post)
        } else {
            Err(ApiError::NotFound(format!("Post with id {} not found", post_id)))
        }
    }

    /// ユーザー ID で絞り込むかどうかを `Option<&str>` で表現している。
    /// `if let Some(...)` で分岐し、SQL をそれぞれ書き換えるパターン。
    pub async fn get_all_posts(&self, user_id_filter: Option<&str>) -> Result<Vec<Post>, ApiError> {
        let client = self.get_connection().await?;
        
        if let Some(user_id_str) = user_id_filter {
            // Parse the user_id string to UUID
            let user_uuid = uuid::Uuid::parse_str(user_id_str)
                .map_err(|_| ApiError::Validation("Invalid user ID format".to_string()))?;
                
            let query = "SELECT id, user_id, title, content, created_at, updated_at FROM posts WHERE user_id = $1 ORDER BY created_at DESC";
            let rows = client.query(query, &[&user_uuid])
                .await
                .map_err(ApiError::from)?;
                
            let posts: Vec<Post> = rows.iter().map(|row| {
                Post {
                    id: row.get(0),
                    user_id: row.get(1),
                    title: row.get(2),
                    content: row.get(3),
                    created_at: row.get(4),
                    updated_at: row.get(5),
                }
            }).collect();
            
            Ok(posts)
        } else {
            let query = "SELECT id, user_id, title, content, created_at, updated_at FROM posts ORDER BY created_at DESC";
            let rows = client.query(query, &[])
                .await
                .map_err(ApiError::from)?;
                
            let posts: Vec<Post> = rows.iter().map(|row| {
                Post {
                    id: row.get(0),
                    user_id: row.get(1),
                    title: row.get(2),
                    content: row.get(3),
                    created_at: row.get(4),
                    updated_at: row.get(5),
                }
            }).collect();
            
            Ok(posts)
        }
    }

    /// 特定ユーザーの投稿のみを取るショートカット。
    /// `get_all_posts` のフィルタ版を明示的に公開している。
    pub async fn get_posts_by_user_id(&self, user_id: &str) -> Result<Vec<Post>, ApiError> {
        // Parse the user_id string to UUID
        let uuid = uuid::Uuid::parse_str(user_id)
            .map_err(|_| ApiError::Validation("Invalid user ID format".to_string()))?;
            
        let client = self.get_connection().await?;
        let query = "SELECT id, user_id, title, content, created_at, updated_at FROM posts WHERE user_id = $1 ORDER BY created_at DESC";
        
        let rows = client.query(query, &[&uuid])
            .await
            .map_err(ApiError::from)?;
        
        let posts: Vec<Post> = rows.iter().map(|row| {
            Post {
                id: row.get(0),
                user_id: row.get(1),
                title: row.get(2),
                content: row.get(3),
                created_at: row.get(4),
                updated_at: row.get(5),
            }
        }).collect();
        
        Ok(posts)
    }

    // Vocabulary repository operations

    /// 語彙データの作成。
    /// 例文フィールドは `Option<String>` なので、`get_normalized_*` で空文字を None に変換している。
    pub async fn create_vocabulary(&self, request: CreateVocabularyRequest) -> Result<Vocabulary, ApiError> {
        // Validate the request
        request.validate().map_err(ApiError::Validation)?;
        
        // Get normalized values
        let en_word = request.get_normalized_en_word();
        let ja_word = request.get_normalized_ja_word();
        let en_example = request.get_normalized_en_example();
        let ja_example = request.get_normalized_ja_example();
        
        let client = self.get_connection().await?;
        
        let query = r#"
            INSERT INTO vocabulary (en_word, ja_word, en_example, ja_example, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            RETURNING id, en_word, ja_word, en_example, ja_example, created_at, updated_at
        "#;
        
        let row = client.query_one(
            query,
            &[&en_word, &ja_word, &en_example, &ja_example]
        )
        .await
        .map_err(ApiError::from)?;
        
        let created_vocabulary = Vocabulary {
            id: row.get(0),
            en_word: row.get(1),
            ja_word: row.get(2),
            en_example: row.get(3),
            ja_example: row.get(4),
            created_at: row.get(5),
            updated_at: row.get(6),
        };
        
        info!("Created vocabulary entry with id: {}", created_vocabulary.id);
        Ok(created_vocabulary)
    }

    /// オートインクリメント ID (i32) でレコードを取得する。
    /// 敢えて UUID ではなく整数を使う例としてわかりやすい。
    pub async fn get_vocabulary_by_id(&self, id: i32) -> Result<Vocabulary, ApiError> {
        let client = self.get_connection().await?;
        let query = "SELECT id, en_word, ja_word, en_example, ja_example, created_at, updated_at FROM vocabulary WHERE id = $1";
        
        let row = client.query_opt(query, &[&id])
            .await
            .map_err(ApiError::from)?;
        
        if let Some(row) = row {
            let vocabulary = Vocabulary {
                id: row.get(0),
                en_word: row.get(1),
                ja_word: row.get(2),
                en_example: row.get(3),
                ja_example: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            
            Ok(vocabulary)
        } else {
            Err(ApiError::NotFound(format!("Vocabulary entry with id {} not found", id)))
        }
    }

    /// 登録順に語彙を列挙する。
    /// `Vec<Vocabulary>` を返すので、ハンドラ側はそのまま JSON 配列にできる。
    pub async fn get_all_vocabulary(&self) -> Result<Vec<Vocabulary>, ApiError> {
        let client = self.get_connection().await?;
        let query = "SELECT id, en_word, ja_word, en_example, ja_example, created_at, updated_at FROM vocabulary ORDER BY created_at DESC";
        
        let rows = client.query(query, &[])
            .await
            .map_err(ApiError::from)?;
        
        let vocabulary_list: Vec<Vocabulary> = rows.iter().map(|row| {
            Vocabulary {
                id: row.get(0),
                en_word: row.get(1),
                ja_word: row.get(2),
                en_example: row.get(3),
                ja_example: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            }
        }).collect();
        
        Ok(vocabulary_list)
    }

    /// 開発用のシードデータを投入する。
    /// 既にレコードが存在する場合は何もしないことで、重複挿入を避けている。
    pub async fn seed_vocabulary(&self) -> Result<(), ApiError> {
        info!("Seeding vocabulary data");
        
        let client = self.get_connection().await?;
        
        // Check if vocabulary table already has data
        let count_query = "SELECT COUNT(*) FROM vocabulary";
        let row = client.query_one(count_query, &[])
            .await
            .map_err(ApiError::from)?;
        let count: i64 = row.get(0);
        
        if count > 0 {
            info!("Vocabulary table already contains {} entries, skipping seed", count);
            return Ok(());
        }
        
        // Seed data
        let seed_data = vec![
            ("apple", "りんご", "I eat an apple every day.", "私は毎日りんごを食べます。"),
            ("book", "本", "This is an interesting book.", "これは面白い本です。"),
            ("computer", "コンピューター", "I use my computer for work.", "私は仕事でコンピューターを使います。"),
            ("study", "勉強する", "I study English every morning.", "私は毎朝英語を勉強します。"),
            ("friend", "友達", "She is my best friend.", "彼女は私の親友です。"),
        ];
        
        let insert_query = r#"
            INSERT INTO vocabulary (en_word, ja_word, en_example, ja_example, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
        "#;
        
        for (en_word, ja_word, en_example, ja_example) in seed_data {
            client.execute(
                insert_query,
                &[&en_word, &ja_word, &en_example, &ja_example]
            )
            .await
            .map_err(ApiError::from)?;
            
            info!("Seeded vocabulary: {} -> {}", en_word, ja_word);
        }
        
        info!("Successfully seeded 5 vocabulary entries");
        Ok(())
    }

    /// `ORDER BY RANDOM()` を使って 1 件ランダム取得するサンプル。
    /// 学習アプリの「出題」機能に応用できる。
    pub async fn get_random_vocabulary(&self) -> Result<Vocabulary, ApiError> {
        let client = self.get_connection().await?;
        let query = "SELECT id, en_word, ja_word, en_example, ja_example, created_at, updated_at FROM vocabulary ORDER BY RANDOM() LIMIT 1";
        
        let row = client.query_opt(query, &[])
            .await
            .map_err(ApiError::from)?;
        
        if let Some(row) = row {
            let vocabulary = Vocabulary {
                id: row.get(0),
                en_word: row.get(1),
                ja_word: row.get(2),
                en_example: row.get(3),
                ja_example: row.get(4),
                created_at: row.get(5),
                updated_at: row.get(6),
            };
            
            Ok(vocabulary)
        } else {
            Err(ApiError::NotFound("No vocabulary entries found".to_string()))
        }
    }
}

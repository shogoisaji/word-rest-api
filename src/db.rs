use crate::error::ApiError;
use crate::models::user::{User, CreateUserRequest, UpdateUserRequest};
use crate::models::post::{Post, CreatePostRequest};
use turso::{Builder, Database as TursoDatabase, Connection};
use std::sync::Arc;
use tracing::{error, info};

/// Database wrapper that manages Turso connections
#[derive(Clone)]
pub struct Database {
    db: Arc<TursoDatabase>,
}

impl Database {
    /// Create a new database connection
    /// 
    /// # Arguments
    /// * `url` - The Turso database URL
    /// * `auth_token` - The authentication token for Turso
    /// 
    /// # Returns
    /// * `Result<Self, ApiError>` - Database instance or error
    pub async fn new(url: &str, auth_token: &str) -> Result<Self, ApiError> {
        info!("Connecting to database at: {}", url);
        
        let db = if url.starts_with("file:") {
            // Local database file
            let path = url.strip_prefix("file:").unwrap_or(url);
            Builder::new_local(path).build().await
        } else {
            // Remote Turso database
            Builder::new_remote(url.to_string(), auth_token.to_string()).build().await
        }.map_err(|e| {
            error!("Failed to create database: {}", e);
            ApiError::Database(e)
        })?;

        Ok(Database { db: Arc::new(db) })
    }

    /// Get a connection from the database
    async fn connection(&self) -> Result<Connection, ApiError> {
        self.db.connect().map_err(|e| {
            error!("Failed to get database connection: {}", e);
            ApiError::Database(e)
        })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<(), ApiError> {
        info!("Running database migrations");
        
        let conn = self.connection().await?;
        
        // Create users table
        let users_table = r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
        "#;
        
        conn.execute(users_table, ())
            .await
            .map_err(|e| {
                error!("Failed to create users table: {}", e);
                ApiError::Database(e)
            })?;

        // Create index on email for users table
        let users_email_index = "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)";
        
        conn.execute(users_email_index, ())
            .await
            .map_err(|e| {
                error!("Failed to create users email index: {}", e);
                ApiError::Database(e)
            })?;

        // Create posts table
        let posts_table = r#"
            CREATE TABLE IF NOT EXISTS posts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
        "#;
        
        conn.execute(posts_table, ())
            .await
            .map_err(|e| {
                error!("Failed to create posts table: {}", e);
                ApiError::Database(e)
            })?;

        // Create index on user_id for posts table
        let posts_user_index = "CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id)";
        
        conn.execute(posts_user_index, ())
            .await
            .map_err(|e| {
                error!("Failed to create posts user_id index: {}", e);
                ApiError::Database(e)
            })?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Test database connection
    pub async fn test_connection(&self) -> Result<(), ApiError> {
        let conn = self.connection().await?;
        
        // Simple query to test connection
        conn.execute("SELECT 1", ())
            .await
            .map_err(|e| {
                error!("Database connection test failed: {}", e);
                ApiError::Database(e)
            })?;
            
        info!("Database connection test successful");
        Ok(())
    }

    // User repository operations

    /// Create a new user
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, ApiError> {
        // Validate the request
        request.validate().map_err(ApiError::Validation)?;
        
        let user = request.into_user();
        let conn = self.connection().await?;
        
        let query = r#"
            INSERT INTO users (id, name, email, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
        "#;
        
        let user_id = user.id.clone();
        
        conn.execute(
            query,
            [&user.id, &user.name, &user.email, &user.created_at.to_string(), &user.updated_at.to_string()]
        )
        .await
        .map_err(|e| {
            // Check for unique constraint violation (email already exists)
            if e.to_string().contains("UNIQUE constraint failed: users.email") {
                ApiError::Conflict("Email already exists".to_string())
            } else {
                error!("Failed to create user: {}", e);
                ApiError::Database(e)
            }
        })?;
        
        info!("Created user with id: {}", user_id);
        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user_by_id(&self, user_id: &str) -> Result<User, ApiError> {
        let conn = self.connection().await?;
        let query = "SELECT id, name, email, created_at, updated_at FROM users WHERE id = ?1";
        
        let mut rows = conn.prepare(query)
            .await
            .map_err(|e| {
                error!("Failed to prepare get user query: {}", e);
                ApiError::Database(e)
            })?
            .query([user_id])
            .await
            .map_err(|e| {
                error!("Failed to execute get user query: {}", e);
                ApiError::Database(e)
            })?;
        
        if let Some(row) = rows.next().await.map_err(|e| {
            error!("Failed to fetch user row: {}", e);
            ApiError::Database(e)
        })? {
            let user = User {
                id: row.get::<String>(0).map_err(|e| {
                    error!("Failed to get user id: {}", e);
                    ApiError::Database(e)
                })?,
                name: row.get::<String>(1).map_err(|e| {
                    error!("Failed to get user name: {}", e);
                    ApiError::Database(e)
                })?,
                email: row.get::<String>(2).map_err(|e| {
                    error!("Failed to get user email: {}", e);
                    ApiError::Database(e)
                })?,
                created_at: row.get::<i64>(3).map_err(|e| {
                    error!("Failed to get user created_at: {}", e);
                    ApiError::Database(e)
                })?,
                updated_at: row.get::<i64>(4).map_err(|e| {
                    error!("Failed to get user updated_at: {}", e);
                    ApiError::Database(e)
                })?,
            };
            
            Ok(user)
        } else {
            Err(ApiError::NotFound(format!("User with id {} not found", user_id)))
        }
    }

    /// Get all users
    pub async fn get_all_users(&self) -> Result<Vec<User>, ApiError> {
        let conn = self.connection().await?;
        let query = "SELECT id, name, email, created_at, updated_at FROM users ORDER BY created_at DESC";
        
        let mut rows = conn.prepare(query)
            .await
            .map_err(|e| {
                error!("Failed to prepare get all users query: {}", e);
                ApiError::Database(e)
            })?
            .query(())
            .await
            .map_err(|e| {
                error!("Failed to execute get all users query: {}", e);
                ApiError::Database(e)
            })?;
        
        let mut users = Vec::new();
        
        while let Some(row) = rows.next().await.map_err(|e| {
            error!("Failed to fetch user row: {}", e);
            ApiError::Database(e)
        })? {
            let user = User {
                id: row.get::<String>(0).map_err(|e| {
                    error!("Failed to get user id: {}", e);
                    ApiError::Database(e)
                })?,
                name: row.get::<String>(1).map_err(|e| {
                    error!("Failed to get user name: {}", e);
                    ApiError::Database(e)
                })?,
                email: row.get::<String>(2).map_err(|e| {
                    error!("Failed to get user email: {}", e);
                    ApiError::Database(e)
                })?,
                created_at: row.get::<i64>(3).map_err(|e| {
                    error!("Failed to get user created_at: {}", e);
                    ApiError::Database(e)
                })?,
                updated_at: row.get::<i64>(4).map_err(|e| {
                    error!("Failed to get user updated_at: {}", e);
                    ApiError::Database(e)
                })?,
            };
            
            users.push(user);
        }
        
        Ok(users)
    }

    /// Update user by ID
    pub async fn update_user(&self, user_id: &str, request: UpdateUserRequest) -> Result<User, ApiError> {
        // Validate the request
        request.validate().map_err(ApiError::Validation)?;
        
        // First, get the existing user
        let mut user = self.get_user_by_id(user_id).await?;
        
        // Update the user with new values
        user.update(request.get_normalized_name(), request.get_normalized_email());
        
        let conn = self.connection().await?;
        let query = r#"
            UPDATE users 
            SET name = ?1, email = ?2, updated_at = ?3
            WHERE id = ?4
        "#;
        
        let user_id = user.id.clone();
        
        conn.execute(
            query,
            [&user.name, &user.email, &user.updated_at.to_string(), &user.id]
        )
        .await
        .map_err(|e| {
            // Check for unique constraint violation (email already exists)
            if e.to_string().contains("UNIQUE constraint failed: users.email") {
                ApiError::Conflict("Email already exists".to_string())
            } else {
                error!("Failed to update user: {}", e);
                ApiError::Database(e)
            }
        })?;
        
        info!("Updated user with id: {}", user_id);
        Ok(user)
    }

    /// Delete user by ID (with cascade delete of posts)
    pub async fn delete_user(&self, user_id: &str) -> Result<(), ApiError> {
        // First check if user exists
        self.get_user_by_id(user_id).await?;
        
        let conn = self.connection().await?;
        
        // Delete user (posts will be cascade deleted due to foreign key constraint)
        let query = "DELETE FROM users WHERE id = ?1";
        
        conn.execute(query, [user_id])
            .await
            .map_err(|e| {
                error!("Failed to delete user: {}", e);
                ApiError::Database(e)
            })?;
        
        info!("Deleted user with id: {} (cascade deleted associated posts)", user_id);
        Ok(())
    }

    // Post repository operations

    /// Create a new post
    pub async fn create_post(&self, request: CreatePostRequest) -> Result<Post, ApiError> {
        // Validate the request
        request.validate().map_err(ApiError::Validation)?;
        
        // Check if user exists
        self.get_user_by_id(&request.user_id).await?;
        
        let post = request.into_post();
        let conn = self.connection().await?;
        
        let query = r#"
            INSERT INTO posts (id, user_id, title, content, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#;
        
        let post_id = post.id.clone();
        let content_str = post.content.as_deref().unwrap_or("");
        
        conn.execute(
            query,
            [&post.id, &post.user_id, &post.title, content_str, &post.created_at.to_string(), &post.updated_at.to_string()]
        )
        .await
        .map_err(|e| {
            // Check for foreign key constraint violation
            if e.to_string().contains("FOREIGN KEY constraint failed") {
                ApiError::Validation("User does not exist".to_string())
            } else {
                error!("Failed to create post: {}", e);
                ApiError::Database(e)
            }
        })?;
        
        info!("Created post with id: {}", post_id);
        Ok(post)
    }

    /// Get post by ID
    pub async fn get_post_by_id(&self, post_id: &str) -> Result<Post, ApiError> {
        let conn = self.connection().await?;
        let query = "SELECT id, user_id, title, content, created_at, updated_at FROM posts WHERE id = ?1";
        
        let mut rows = conn.prepare(query)
            .await
            .map_err(|e| {
                error!("Failed to prepare get post query: {}", e);
                ApiError::Database(e)
            })?
            .query([post_id])
            .await
            .map_err(|e| {
                error!("Failed to execute get post query: {}", e);
                ApiError::Database(e)
            })?;
        
        if let Some(row) = rows.next().await.map_err(|e| {
            error!("Failed to fetch post row: {}", e);
            ApiError::Database(e)
        })? {
            let content_str: Option<String> = row.get(3).map_err(|e| {
                error!("Failed to get post content: {}", e);
                ApiError::Database(e)
            })?;
            
            let post = Post {
                id: row.get::<String>(0).map_err(|e| {
                    error!("Failed to get post id: {}", e);
                    ApiError::Database(e)
                })?,
                user_id: row.get::<String>(1).map_err(|e| {
                    error!("Failed to get post user_id: {}", e);
                    ApiError::Database(e)
                })?,
                title: row.get::<String>(2).map_err(|e| {
                    error!("Failed to get post title: {}", e);
                    ApiError::Database(e)
                })?,
                content: if content_str.as_deref() == Some("") { None } else { content_str },
                created_at: row.get::<i64>(4).map_err(|e| {
                    error!("Failed to get post created_at: {}", e);
                    ApiError::Database(e)
                })?,
                updated_at: row.get::<i64>(5).map_err(|e| {
                    error!("Failed to get post updated_at: {}", e);
                    ApiError::Database(e)
                })?,
            };
            
            Ok(post)
        } else {
            Err(ApiError::NotFound(format!("Post with id {} not found", post_id)))
        }
    }

    /// Get all posts, optionally filtered by user_id
    pub async fn get_all_posts(&self, user_id_filter: Option<&str>) -> Result<Vec<Post>, ApiError> {
        let conn = self.connection().await?;
        
        let mut rows = if let Some(user_id) = user_id_filter {
            // Verify user exists if filtering by user_id
            self.get_user_by_id(user_id).await?;
            
            let query = "SELECT id, user_id, title, content, created_at, updated_at FROM posts WHERE user_id = ?1 ORDER BY created_at DESC";
            conn.prepare(query)
                .await
                .map_err(|e| {
                    error!("Failed to prepare get posts by user query: {}", e);
                    ApiError::Database(e)
                })?
                .query([user_id])
                .await
                .map_err(|e| {
                    error!("Failed to execute get posts by user query: {}", e);
                    ApiError::Database(e)
                })?
        } else {
            let query = "SELECT id, user_id, title, content, created_at, updated_at FROM posts ORDER BY created_at DESC";
            conn.prepare(query)
                .await
                .map_err(|e| {
                    error!("Failed to prepare get all posts query: {}", e);
                    ApiError::Database(e)
                })?
                .query(())
                .await
                .map_err(|e| {
                    error!("Failed to execute get all posts query: {}", e);
                    ApiError::Database(e)
                })?
        };
        
        let mut posts = Vec::new();
        
        while let Some(row) = rows.next().await.map_err(|e| {
            error!("Failed to fetch post row: {}", e);
            ApiError::Database(e)
        })? {
            let content_str: Option<String> = row.get(3).map_err(|e| {
                error!("Failed to get post content: {}", e);
                ApiError::Database(e)
            })?;
            
            let post = Post {
                id: row.get::<String>(0).map_err(|e| {
                    error!("Failed to get post id: {}", e);
                    ApiError::Database(e)
                })?,
                user_id: row.get::<String>(1).map_err(|e| {
                    error!("Failed to get post user_id: {}", e);
                    ApiError::Database(e)
                })?,
                title: row.get::<String>(2).map_err(|e| {
                    error!("Failed to get post title: {}", e);
                    ApiError::Database(e)
                })?,
                content: if content_str.as_deref() == Some("") { None } else { content_str },
                created_at: row.get::<i64>(4).map_err(|e| {
                    error!("Failed to get post created_at: {}", e);
                    ApiError::Database(e)
                })?,
                updated_at: row.get::<i64>(5).map_err(|e| {
                    error!("Failed to get post updated_at: {}", e);
                    ApiError::Database(e)
                })?,
            };
            
            posts.push(post);
        }
        
        Ok(posts)
    }

    /// Get posts by user ID
    pub async fn get_posts_by_user_id(&self, user_id: &str) -> Result<Vec<Post>, ApiError> {
        self.get_all_posts(Some(user_id)).await
    }
}
# 本番環境デプロイガイド

このガイドでは、Rust PostgreSQL APIをGoogle Cloud Runに本番環境レベルの設定、監視、セキュリティのベストプラクティスとともにデプロイする方法を包括的に説明します。

## 🚀 クイックスタート

迅速なデプロイのために、以下の手順に従ってください：

```bash
# 1. Google Cloudリソースのセットアップ
# 2. データベース認証情報の設定
# 3. アプリケーションのデプロイ
# 4. ヘルスチェックの実行
```

## 📋 前提条件

### 必要なツール
- **Google Cloud CLI** (gcloud) - [インストールガイド](https://cloud.google.com/sdk/docs/install)
- **Docker** - [インストールガイド](https://docs.docker.com/get-docker/)
- **Git** - バージョン管理用
- **curl** と **jq** - テストとヘルスチェック用

### 必要なアカウントとサービス
- **Google Cloudプロジェクト** - 請求が有効化されていること
- **PostgreSQLデータベース** (Neon推奨) - [Neonセットアップ](https://neon.tech/)

### 前提条件の確認
```bash
# ツールのバージョン確認
gcloud version
docker --version
git --version
curl --version
jq --version

# Google Cloudで認証
gcloud auth login
gcloud auth application-default login
```


## 🗄️ データベースセットアップ (PostgreSQL/Neon)

### 1. Neonデータベースの作成（推奨）

#### オプション1: Neonコンソールを使用（推奨）
1. https://neon.tech にアクセスしてアカウントを作成
2. 新しいプロジェクトを作成
3. データベースを作成（デフォルト: `neondb`）
4. ダッシュボードから接続文字列をコピー
5. 接続文字列の形式：
   ```
   postgresql://username:password@ep-xxxxx-xxxxx.region.aws.neon.tech/neondb?sslmode=require
   ```

**重要な注意事項：**
- **Pooled connection**の接続文字列を使用してください（Cloud Run推奨）
- 接続文字列に`sslmode=require`が含まれていることを確認
- 接続文字列を安全に保存 - デプロイ時に必要になります

### 2. 代替案: ローカルPostgreSQLセットアップ

ローカル開発またはテスト用：

```bash
# PostgreSQLのインストール（macOS）
brew install postgresql

# またはUbuntu/Debian
sudo apt-get install postgresql postgresql-contrib

# PostgreSQLサービスの開始
brew services start postgresql  # macOS
# または
sudo systemctl start postgresql  # Ubuntu

# データベースとユーザーの作成
createdb word_rest_api
createuser -s word_user
psql -c "ALTER USER word_user PASSWORD 'your_secure_password';"
```

### 3. データベース接続のテスト

```bash
# psqlを使用して接続をテスト
psql "postgresql://username:password@host:port/database?sslmode=require"

# または接続パラメータでテスト
psql -h host -p port -U username -d database

# 接続が成功したことを確認
# psqlプロンプトが表示されます: database=>
# \q で終了
```

### 4. データベーススキーマ

アプリケーションは起動時に自動的にテーブルを作成します。スキーマには以下が含まれます：
- UUIDプライマリキーとPostgreSQLタイムスタンプを持つ`users`テーブル
- 外部キー関係とCASCADE削除を持つ`posts`テーブル
- パフォーマンス用の適切なインデックス（email、user_id、created_at）
- 自動UUID生成のためのUUID拡張機能が有効化


## ☁️ Google Cloudセットアップ

### 1. プロジェクトの作成または選択

```bash
# 既存のプロジェクトを一覧表示
gcloud projects list

# 新しいプロジェクトを作成（必要な場合）
# 注意: プロジェクトIDはグローバルに一意である必要があります
gcloud projects create YOUR-PROJECT-ID --name="Word REST API"

# プロジェクトを設定
export PROJECT_ID="YOUR-PROJECT-ID"
gcloud config set project $PROJECT_ID
```

### 2. 請求の有効化

Cloud Runやその他のサービスには請求の有効化が必要です：

1. https://console.cloud.google.com/billing にアクセス
2. プロジェクトに請求アカウントをリンク
3. またはCLIを使用：
   ```bash
   # 請求アカウントを一覧表示
   gcloud billing accounts list
   
   # プロジェクトに請求アカウントをリンク
   gcloud billing projects link $PROJECT_ID --billing-account=BILLING-ACCOUNT-ID
   ```

### 3. 必要なAPIの有効化

```bash
# すべての必要なAPIを有効化
gcloud services enable run.googleapis.com
gcloud services enable cloudbuild.googleapis.com
gcloud services enable artifactregistry.googleapis.com
gcloud services enable secretmanager.googleapis.com

# APIが有効化されたことを確認
gcloud services list --enabled
```

### 4. Artifact Registryリポジトリの作成

```bash
# Dockerリポジトリを作成
gcloud artifacts repositories create word-rest-api \
  --repository-format=docker \
  --location=asia-northeast1 \
  --description="Word REST API Docker images"

# リポジトリが作成されたことを確認
gcloud artifacts repositories list

# Docker認証を設定
gcloud auth configure-docker asia-northeast1-docker.pkg.dev
```


## 🔐 シークレット管理

### PostgreSQL/Neon認証情報をSecret Managerに保存

**重要**: データベース認証情報をバージョン管理にコミットしないでください！

```bash
# データベース接続文字列を保存
# 実際の接続文字列に置き換えてください
echo -n "postgresql://username:password@host:port/database?sslmode=require" | \
  gcloud secrets create database-url --data-file=-

# シークレットが作成されたことを確認
gcloud secrets list

# シークレットのメタデータを表示（実際の値ではありません）
gcloud secrets describe database-url
```

### Cloud Runにシークレットアクセスを付与

```bash
# プロジェクト番号を取得
PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")

# デフォルトのコンピュートサービスアカウントにSecret Managerアクセスを付与
gcloud secrets add-iam-policy-binding database-url \
  --member="serviceAccount:${PROJECT_NUMBER}-compute@developer.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"

# 権限を確認
gcloud secrets get-iam-policy database-url
```

### シークレットの更新（必要な場合）

```bash
# シークレットの新しいバージョンを追加
echo -n "new-connection-string" | \
  gcloud secrets versions add database-url --data-file=-

# すべてのバージョンを一覧表示
gcloud secrets versions list database-url

# 特定のバージョンにアクセス
gcloud secrets versions access 2 --secret=database-url
```


## 🚢 デプロイ

### 1. デプロイの準備

```bash
# プロジェクトのルートディレクトリにいることを確認
cd /path/to/word_rest_api

# Dockerfileが存在することを確認
ls -la Dockerfile

# ソースコードの準備ができていることを確認
cargo check
```

### 2. Cloud Runへのデプロイ

#### ソースベースデプロイ（推奨）

この方法では、ソースコードから自動的にDockerイメージをビルドします：

```bash
gcloud run deploy word-rest-api \
  --source . \
  --region asia-northeast1 \
  --platform managed \
  --allow-unauthenticated \
  --set-secrets="DATABASE_URL=database-url:latest" \
  --set-env-vars="ENV=production,RUST_LOG=info" \
  --memory 512Mi \
  --cpu 1 \
  --max-instances 10 \
  --min-instances 0 \
  --timeout 300
```

**デプロイパラメータの説明：**
- `--source .`: 現在のディレクトリからビルド
- `--region asia-northeast1`: 東京リージョンにデプロイ（必要に応じて変更）
- `--allow-unauthenticated`: パブリックアクセスを許可（プライベートAPIの場合は削除）
- `--set-secrets`: Secret ManagerからデータベースURLをマウント
- `--set-env-vars`: 環境変数を設定
- `--memory 512Mi`: 512MBのRAMを割り当て
- `--cpu 1`: 1 vCPUを割り当て
- `--max-instances 10`: 最大同時インスタンス数
- `--min-instances 0`: アイドル時にゼロにスケール（コスト最適化）
- `--timeout 300`: リクエストタイムアウト（秒）

#### 手動Dockerビルド＆デプロイ（代替案）

Dockerイメージを手動でビルドする場合：

```bash
# Dockerイメージをビルド
docker build -t asia-northeast1-docker.pkg.dev/$PROJECT_ID/word-rest-api/word-rest-api:latest .

# Artifact Registryにプッシュ
docker push asia-northeast1-docker.pkg.dev/$PROJECT_ID/word-rest-api/word-rest-api:latest

# Cloud Runにデプロイ
gcloud run deploy word-rest-api \
  --image asia-northeast1-docker.pkg.dev/$PROJECT_ID/word-rest-api/word-rest-api:latest \
  --region asia-northeast1 \
  --platform managed \
  --allow-unauthenticated \
  --set-secrets="DATABASE_URL=database-url:latest" \
  --set-env-vars="ENV=production,RUST_LOG=info" \
  --memory 512Mi \
  --cpu 1 \
  --max-instances 10 \
  --min-instances 0 \
  --timeout 300
```

### 3. デプロイ時間

- **初回デプロイ**: 5-10分（Rustアプリケーションのビルドを含む）
- **以降のデプロイ**: 3-5分（Dockerレイヤーキャッシュあり）
- **Rustコンパイル**: 最も時間がかかるステップ

### 4. サービスURLの取得

```bash
# デプロイされたサービスURLを取得
SERVICE_URL=$(gcloud run services describe word-rest-api \
  --region=asia-northeast1 \
  --format='value(status.url)')

echo "Service URL: $SERVICE_URL"
```


## 🏥 ヘルスチェックと検証

### 1. 基本的なヘルスチェック

```bash
# ヘルスチェックエンドポイント
curl $SERVICE_URL/health

# 期待されるレスポンス: OK
```

### 2. APIエンドポイントのテスト

```bash
# すべてのユーザーを取得（初期状態では空の配列）
curl $SERVICE_URL/api/users

# テストユーザーを作成
curl -X POST $SERVICE_URL/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Test User", "email": "test@example.com"}'

# IDでユーザーを取得（前のレスポンスの実際のIDに置き換え）
curl $SERVICE_URL/api/users/USER-ID-HERE

# すべてのユーザーを取得（作成したユーザーが表示されるはず）
curl $SERVICE_URL/api/users

# 投稿を作成
curl -X POST $SERVICE_URL/api/posts \
  -H "Content-Type: application/json" \
  -d '{"user_id": "USER-ID-HERE", "title": "Test Post", "content": "Hello World"}'

# すべての投稿を取得
curl $SERVICE_URL/api/posts
```

### 3. ログの表示

```bash
# リアルタイムログをストリーム
gcloud run services logs tail word-rest-api --region=asia-northeast1

# 最近のログを読む
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=word-rest-api" \
  --limit=50 \
  --format=json

# 重要度でログをフィルタ
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=word-rest-api AND severity>=ERROR" \
  --limit=20
```

### 4. サービスステータスの監視

```bash
# サービスの詳細を取得
gcloud run services describe word-rest-api --region=asia-northeast1

# サービスメトリクスを確認
gcloud run services describe word-rest-api \
  --region=asia-northeast1 \
  --format="table(status.url, status.conditions[0].type, status.conditions[0].status)"
```


## 🔧 設定

### 環境変数

| 変数 | 必須 | デフォルト | 説明 |
|------|------|-----------|------|
| `ENV` | いいえ | `local` | 環境（`local`、`production`） |
| `PORT` | いいえ | `8080` | HTTPサーバーポート（Cloud Runが設定） |
| `RUST_LOG` | いいえ | `info` | ログレベル（`error`、`warn`、`info`、`debug`、`trace`） |
| `DATABASE_URL` | はい | - | PostgreSQL接続文字列（Secret Managerから） |
| `DATABASE_HOST` | はい* | - | PostgreSQLホスト（DATABASE_URLの代替） |
| `DATABASE_USERNAME` | はい* | - | PostgreSQLユーザー名（DATABASE_URLの代替） |
| `DATABASE_PASSWORD` | はい* | - | PostgreSQLパスワード（DATABASE_URLの代替） |
| `DATABASE_NAME` | はい* | - | PostgreSQLデータベース名（DATABASE_URLの代替） |
| `DATABASE_SSL_MODE` | いいえ | `require` | SSLモード（disable、allow、prefer、require、verify-ca、verify-full） |
| `DATABASE_MAX_CONNECTIONS` | いいえ | `10` | プール内の最大接続数 |
| `DATABASE_CONNECTION_TIMEOUT` | いいえ | `30` | 接続タイムアウト（秒） |

*`DATABASE_URL`または個別のデータベースパラメータのいずれかが必要です。

### Cloud Run設定

| 設定 | 値 | 説明 |
|------|-----|------|
| メモリ | 512Mi | Rustアプリケーションに十分 |
| CPU | 1 vCPU | 同時リクエストに適したパフォーマンス |
| 同時実行数 | 80 | インスタンスあたりのリクエスト数（デフォルト） |
| 最小インスタンス | 0 | コスト最適化（ゼロにスケール） |
| 最大インスタンス | 10 | 自動スケーリング制限 |
| タイムアウト | 300秒 | リクエストタイムアウト |
| リージョン | asia-northeast1 | 東京（必要に応じて変更） |

### 推奨リージョン

ユーザーに近いリージョンを選択：
- `asia-northeast1` - 東京、日本
- `asia-northeast2` - 大阪、日本
- `us-central1` - アイオワ、米国
- `us-east1` - サウスカロライナ、米国
- `europe-west1` - ベルギー
- `asia-southeast1` - シンガポール


## 🔒 セキュリティのベストプラクティス

### 1. シークレット管理
- ✅ 機密データにはGoogle Secret Managerを使用
- ✅ シークレットをバージョン管理にコミットしない
- ✅ データベースパスワードを定期的にローテーション
- ✅ 最小権限のサービスアカウントを使用
- ✅ ロールバック機能のためにシークレットのバージョニングを有効化

### 2. コンテナセキュリティ
- ✅ 非rootユーザーとして実行（Dockerfileで設定済み）
- ✅ 最小限のベースイメージ（Alpine Linux）
- ✅ 不要な依存関係なし
- ✅ イメージサイズを削減するマルチステージビルド
- ✅ 定期的なセキュリティアップデート

### 3. ネットワークセキュリティ
- ✅ HTTPSをCloud Runが強制（自動）
- ✅ CORSが適切に設定されている
- ✅ リクエストタイムアウトでDoSを防止
- ✅ すべてのエンドポイントで入力検証
- ✅ レート制限（Cloud Armorの追加を検討）

### 4. データベースセキュリティ
- ✅ TLS/SSL接続が必須（Neonが強制）
- ✅ 安全な認証情報での接続プーリング
- ✅ SQLインジェクション防止（パラメータ化クエリ）
- ✅ データベースユーザーの最小権限の原則
- ✅ 定期的なバックアップ（Neonが自動バックアップを提供）

### 5. アプリケーションセキュリティ
- ✅ エラーレスポンスに機密データを含めない
- ✅ シークレットなしの構造化ログ
- ✅ 適切なエラーハンドリング
- ✅ 入力検証とサニタイゼーション
- ✅ UUIDベースの識別子（連番IDではない）


## 🚨 トラブルシューティング

### よくある問題

#### 1. データベース接続の失敗

**症状：**
- "Database connection unavailable"エラー
- "TLS handshake failed"エラー
- アプリケーションの起動失敗

**解決方法：**
```bash
# シークレットに正しい接続文字列が含まれているか確認
gcloud secrets versions access latest --secret=database-url

# ローカルで接続をテスト
psql "$(gcloud secrets versions access latest --secret=database-url)"

# 接続文字列にsslmode=requireが含まれているか確認
# Neonはssl接続が必須です

# サービスアカウントがシークレットにアクセスできるか確認
gcloud secrets get-iam-policy database-url
```

#### 2. ビルドの失敗

**症状：**
- "Build failed; check build logs for details"
- Rustコンパイルエラー
- Dockerビルドエラー

**解決方法：**
```bash
# ビルドログを確認
gcloud builds list --region=asia-northeast1 --limit=5

# 詳細なビルドログを表示
BUILD_ID=$(gcloud builds list --region=asia-northeast1 --limit=1 --format="value(id)")
gcloud builds log $BUILD_ID --region=asia-northeast1

# ローカルでビルドをテスト
docker build -t test-image .

# Cargo.lockが.dockerignoreに含まれていないか確認
cat .dockerignore | grep -v "^#" | grep Cargo.lock
```

#### 3. デプロイタイムアウト

**症状：**
- "Container failed to start within allocated timeout"
- ヘルスチェックの失敗

**解決方法：**
```bash
# 起動タイムアウトを増やす
gcloud run services update word-rest-api \
  --region=asia-northeast1 \
  --timeout=600

# 起動時の問題についてコンテナログを確認
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=word-rest-api" \
  --limit=50

# データベースがCloud Runからアクセス可能か確認
# Neonダッシュボードで接続制限を確認
```

#### 4. 権限拒否エラー

**症状：**
- "Permission denied on secret"
- "Failed to access Secret Manager"

**解決方法：**
```bash
# サービスアカウントにシークレットアクセスを付与
PROJECT_NUMBER=$(gcloud projects describe $PROJECT_ID --format="value(projectNumber)")

gcloud secrets add-iam-policy-binding database-url \
  --member="serviceAccount:${PROJECT_NUMBER}-compute@developer.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"

# 権限を確認
gcloud secrets get-iam-policy database-url
```

#### 5. 高メモリ使用量

**症状：**
- コンテナの再起動
- メモリ不足エラー

**解決方法：**
```bash
# メモリ割り当てを増やす
gcloud run services update word-rest-api \
  --region=asia-northeast1 \
  --memory=1Gi

# メモリ使用量を監視
gcloud run services describe word-rest-api \
  --region=asia-northeast1 \
  --format="value(status.url)"

# メモリ関連のエラーについてログを確認
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=word-rest-api AND textPayload=~'memory'" \
  --limit=20
```


## 🔄 メンテナンスと更新

### 1. ローリングアップデート

```bash
# 新しいバージョンをデプロイ（自動ローリングアップデート）
gcloud run deploy word-rest-api \
  --source . \
  --region asia-northeast1

# トラフィックは自動的に新しいリビジョンに移行
# 古いリビジョンはロールバック用に保持
```

### 2. ロールバック

```bash
# リビジョンを一覧表示
gcloud run revisions list --service=word-rest-api --region=asia-northeast1

# 前のリビジョンにロールバック
gcloud run services update-traffic word-rest-api \
  --region=asia-northeast1 \
  --to-revisions=PREVIOUS-REVISION-NAME=100
```

### 3. データベースマイグレーション

```bash
# マイグレーションは起動時に自動実行
# マイグレーションステータスについてログを確認
gcloud logging read "resource.type=cloud_run_revision AND resource.labels.service_name=word-rest-api AND textPayload=~'migration'" \
  --limit=20

# 手動マイグレーションの場合、データベースに接続
psql "$(gcloud secrets versions access latest --secret=database-url)"
```

### 4. 監視とアラート

```bash
# Cloud Consoleでサービスメトリクスを表示
# https://console.cloud.google.com/run

# アップタイムチェックを設定
# https://console.cloud.google.com/monitoring/uptime

# 以下のアラートポリシーを作成：
# - 高エラー率
# - 高レイテンシ
# - 低可用性
# - 高メモリ使用量
```

### 5. バックアップとリカバリ

```bash
# Neonは自動バックアップを提供
# バックアップ設定についてNeonダッシュボードを確認

# 手動データベースエクスポート
pg_dump "$(gcloud secrets versions access latest --secret=database-url)" > backup.sql

# バックアップから復元
psql "$(gcloud secrets versions access latest --secret=database-url)" < backup.sql

# Cloud Storageにエクスポート（大規模データベース推奨）
pg_dump "$(gcloud secrets versions access latest --secret=database-url)" | \
  gzip | \
  gsutil cp - gs://YOUR-BUCKET/backups/backup-$(date +%Y%m%d-%H%M%S).sql.gz
```


## 📈 スケーリングの考慮事項

### 水平スケーリング
- Cloud Runはトラフィックに基づいて自動スケール
- 予想される負荷に基づいて最大インスタンス数を設定
- コールドスタート時間を監視（Rustは高速なコールドスタート）
- 高トラフィックアプリケーションではmin-instances > 0を検討

### データベーススケーリング
- Neonは自動スケーリングを提供
- 接続プールの使用状況を監視
- 読み取り負荷の高いワークロードには読み取りレプリカを検討
- 必要に応じてクエリを最適化しインデックスを追加

### コスト最適化
- コスト削減のためmin-instances=0を使用（ゼロにスケール）
- Cloud Billingで使用状況を監視
- コンテナリソース割り当てを最適化
- Cloud Runの無料枠を使用（月200万リクエスト）
- 予測可能なワークロードには確約利用割引を検討

## 🆘 サポートとリソース

### ドキュメント
- [Cloud Runドキュメント](https://cloud.google.com/run/docs)
- [Neonドキュメント](https://neon.tech/docs)
- [PostgreSQLドキュメント](https://www.postgresql.org/docs/)
- [Axumドキュメント](https://docs.rs/axum/)
- [Tokioドキュメント](https://tokio.rs/)

### 監視
- Google Cloud ConsoleのCloud Runメトリクス
- `gcloud logging`経由のアプリケーションログ
- tracingによるカスタムメトリクス
- データベースメトリクス用のNeonダッシュボード

### ヘルプの取得
- まずサービスログを確認
- ヘルスチェックを実行
- このトラブルシューティングガイドを確認
- GitHubのissuesを確認
- Google Cloudサポート（サポートプランがある場合）

---

## 📝 デプロイチェックリスト

### デプロイ前

- [ ] PostgreSQLデータベース（Neon）が作成され、アクセス可能
- [ ] データベース接続文字列を取得
- [ ] 請求が有効化されたGoogle Cloudプロジェクトを設定
- [ ] 必要なAPIを有効化（Cloud Run、Cloud Build、Artifact Registry、Secret Manager）
- [ ] データベース認証情報をSecret Managerに保存
- [ ] サービスアカウントにSecret Managerアクセスを付与
- [ ] Dockerをインストールして設定
- [ ] ローカルテストが正常に完了
- [ ] コードをバージョン管理にコミット

### デプロイ中

- [ ] ソースコードが正常にアップロード
- [ ] Dockerイメージがエラーなくビルド
- [ ] コンテナがArtifact Registryにプッシュ
- [ ] Cloud Runサービスが作成
- [ ] 環境変数が設定
- [ ] シークレットが正しくマウント
- [ ] サービスが正常にデプロイ

### デプロイ後

- [ ] ヘルスチェックエンドポイントが応答（GET /health）
- [ ] APIエンドポイントが正しく応答
- [ ] データベース操作が動作（作成、読み取り、更新、削除）
- [ ] ログが構造化され読みやすい
- [ ] ログにエラーメッセージがない
- [ ] パフォーマンスが要件を満たす
- [ ] セキュリティのベストプラクティスに従っている
- [ ] 監視が設定されている
- [ ] バックアップ戦略が整っている
- [ ] サービスURLでドキュメントを更新

---

## 🎯 本番環境準備

### セキュリティチェックリスト
- [ ] シークレットがSecret Managerに保存（コードではない）
- [ ] データベース接続でTLS/SSLが有効
- [ ] HTTPSが強制（Cloud Runで自動）
- [ ] すべてのエンドポイントで入力検証
- [ ] エラーメッセージが機密情報を公開しない
- [ ] サービスアカウントが最小権限の原則に従う
- [ ] 定期的なセキュリティアップデートを計画

### パフォーマンスチェックリスト
- [ ] 接続プーリングが設定されている
- [ ] データベースインデックスが作成されている
- [ ] 適切なメモリ/CPU割り当て
- [ ] コールドスタート時間が許容範囲
- [ ] レスポンス時間がSLAを満たす
- [ ] 負荷テストが完了

### 信頼性チェックリスト
- [ ] ヘルスチェックエンドポイントが実装されている
- [ ] グレースフルシャットダウンが設定されている
- [ ] データベースマイグレーションが自動化されている
- [ ] ロールバック手順がテスト済み
- [ ] 監視とアラートが設定されている
- [ ] バックアップとリカバリがテスト済み
- [ ] 災害復旧計画が文書化されている

---

Rust、Axum、PostgreSQL（Neon）、Google Cloud Runで❤️を込めて構築

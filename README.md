# Rust REST API with Axum + PostgreSQL (Neon) + Cloud Run

A high-performance REST API built with Rust, using Axum web framework, PostgreSQL database hosted on Neon, and deployed on Google Cloud Run.

## 🚀 Features

- **Fast & Efficient**: Built with Rust and Axum for maximum performance
- **PostgreSQL Database**: Neon serverless PostgreSQL for scalable data storage
- **Cloud Native**: Designed for Google Cloud Run with auto-scaling
- **Connection Pooling**: Efficient database connection management
- **Structured Logging**: JSON logging with tracing for observability
- **Error Handling**: Comprehensive error handling with proper HTTP status codes
- **Health Monitoring**: Built-in health check endpoint
- **CORS Support**: Cross-origin request handling
- **Graceful Shutdown**: Proper signal handling for container environments

## 📋 API Endpoints

### Health Check
- `GET /health` - Returns service health status

### User Management
- `POST /api/users` - Create a new user
- `GET /api/users` - List all users
- `GET /api/users/:id` - Get user by ID
- `PUT /api/users/:id` - Update user
- `DELETE /api/users/:id` - Delete user (cascades to posts)

### Post Management
- `POST /api/posts` - Create a new post
- `GET /api/posts` - List all posts
- `GET /api/posts/:id` - Get post by ID
- `GET /api/posts?user_id=<id>` - List posts filtered by user

## 🛠 Technology Stack

- **Language**: Rust 2021 Edition
- **Web Framework**: Axum 0.7
- **Async Runtime**: Tokio
- **Database**: PostgreSQL (Neon serverless)
- **Database Driver**: tokio-postgres + deadpool-postgres
- **Serialization**: Serde
- **Logging**: tracing + tracing-subscriber
- **Error Handling**: thiserror + anyhow
- **UUID Generation**: uuid v4
- **Deployment**: Docker + Google Cloud Run

## 📦 Project Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── error.rs             # Error types and handling
├── db.rs                # Database connection and operations
├── middleware.rs        # HTTP middleware (CORS, logging)
├── models/
│   ├── mod.rs
│   ├── user.rs          # User model and validation
│   └── post.rs          # Post model and validation
└── handlers/
    ├── mod.rs
    ├── users.rs         # User CRUD handlers
    └── posts.rs         # Post CRUD handlers
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.80+ installed
- PostgreSQL database (Neon account recommended)
- Docker (for containerization)
- Google Cloud SDK (for deployment)

### 1. Database Setup (Neon PostgreSQL)

```bash
# Option 1: Create database on Neon (recommended)
# 1. Go to https://neon.tech and create an account
# 2. Create a new project and database
# 3. Copy the connection string from the dashboard
# 4. The connection string format is:
#    postgresql://username:password@ep-example-123456.us-east-1.aws.neon.tech/neondb?sslmode=require

# Option 2: Local PostgreSQL setup
# Install PostgreSQL locally
brew install postgresql  # macOS
# or
sudo apt-get install postgresql postgresql-contrib  # Ubuntu

# Start PostgreSQL service
brew services start postgresql  # macOS
# or
sudo systemctl start postgresql  # Ubuntu

# Create database and user
createdb word_rest_api
createuser -s word_user
```

### 2. Local Development

```bash
# Clone the repository
git clone <repository-url>
cd word-rest-api

# Copy environment template
cp .env.example .env

# Edit .env with your PostgreSQL/Neon credentials
# DATABASE_URL=postgresql://username:password@ep-example-123456.us-east-1.aws.neon.tech/neondb?sslmode=require
# Or use individual parameters:
# DATABASE_HOST=ep-example-123456.us-east-1.aws.neon.tech
# DATABASE_USERNAME=username
# DATABASE_PASSWORD=password
# DATABASE_NAME=neondb

# Install dependencies and run
cargo build
cargo run
```

The server will start on `http://localhost:8080`

### 3. Test the API

```bash
# Health check
curl http://localhost:8080/health

# Create a user
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "John Doe", "email": "john@example.com"}'

# Get all users
curl http://localhost:8080/api/users

# Create a post
curl -X POST http://localhost:8080/api/posts \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user-uuid-here", "title": "Hello World", "content": "My first post"}'
```

## 🐳 Docker

### Build and Run Locally

```bash
# Build the Docker image
docker build -t word-rest-api:latest .

# Run the container
docker run -p 8080:8080 \
  -e DATABASE_URL="postgresql://username:password@host:port/database?sslmode=require" \
  word-rest-api:latest
```

## ☁️ Cloud Run Deployment

### 🚀 GitHub Actions (推奨)

このプロジェクトはGitHub Actionsを使った自動デプロイに対応しています。

#### セットアップ

詳細なセットアップ手順は [.github/SETUP.md](.github/SETUP.md) を参照してください。

**クイックスタート:**

1. Google Cloudプロジェクトを作成
2. サービスアカウントを作成し、キーをダウンロード
3. GitHubシークレットを設定:
   - `GCP_PROJECT_ID`: Google CloudプロジェクトID
   - `GCP_SA_KEY`: サービスアカウントキー（JSON）
4. コードをpush:
   ```bash
   git push origin main  # 本番環境にデプロイ
   git push origin develop  # ステージング環境にデプロイ
   ```

#### デプロイワークフロー

- **本番環境**: `main`または`master`ブランチへのpushで自動デプロイ
- **ステージング環境**: `develop`または`staging`ブランチへのpushで自動デプロイ
- **手動デプロイ**: GitHub ActionsのUIから手動トリガー可能

### 🔧 手動デプロイ

#### Prerequisites

1. Google Cloud Project with billing enabled
2. Required APIs enabled:
   - Cloud Run API
   - Artifact Registry API
   - Secret Manager API

#### 1. Setup Google Cloud

```bash
# Set your project ID
export PROJECT_ID="your-project-id"
gcloud config set project $PROJECT_ID

# Enable required APIs
gcloud services enable run.googleapis.com
gcloud services enable artifactregistry.googleapis.com
gcloud services enable secretmanager.googleapis.com

# Create Artifact Registry repository
gcloud artifacts repositories create word-rest-api \
  --repository-format=docker \
  --location=asia-northeast1
```

#### 2. Store Secrets

```bash
# Store PostgreSQL/Neon credentials in Secret Manager
echo -n "postgresql://username:password@host:port/database?sslmode=require" | gcloud secrets create database-url --data-file=-
```

#### 3. Deploy to Cloud Run

```bash
# Deploy using source-based deployment
gcloud run deploy word-rest-api \
  --source . \
  --region asia-northeast1 \
  --platform managed \
  --allow-unauthenticated \
  --set-secrets="DATABASE_URL=database-url:latest" \
  --set-env-vars="ENV=production" \
  --memory 512Mi \
  --cpu 1 \
  --max-instances 10 \
  --min-instances 0 \
  --timeout 300
```

#### 4. Alternative: Manual Docker Deployment

```bash
# Configure Docker for Artifact Registry
gcloud auth configure-docker asia-northeast1-docker.pkg.dev

# Build and push image
docker build -t asia-northeast1-docker.pkg.dev/$PROJECT_ID/word-rest-api/word-rest-api:latest .
docker push asia-northeast1-docker.pkg.dev/$PROJECT_ID/word-rest-api/word-rest-api:latest

# Deploy the image
gcloud run deploy word-rest-api \
  --image asia-northeast1-docker.pkg.dev/$PROJECT_ID/word-rest-api/word-rest-api:latest \
  --region asia-northeast1 \
  --platform managed \
  --allow-unauthenticated \
  --set-secrets="DATABASE_URL=database-url:latest" \
  --set-env-vars="ENV=production" \
  --memory 512Mi \
  --cpu 1 \
  --max-instances 10 \
  --min-instances 0
```

## 🔧 Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PORT` | No | `8080` | HTTP server port |
| `DATABASE_URL` | Yes* | - | PostgreSQL connection string (postgresql://user:pass@host:port/db?sslmode=require) |
| `DATABASE_HOST` | Yes* | `localhost` | PostgreSQL host (alternative to DATABASE_URL) |
| `DATABASE_PORT` | No | `5432` | PostgreSQL port |
| `DATABASE_NAME` | Yes* | - | PostgreSQL database name |
| `DATABASE_USERNAME` | Yes* | - | PostgreSQL username |
| `DATABASE_PASSWORD` | Yes* | - | PostgreSQL password |
| `DATABASE_SSL_MODE` | No | `require` | SSL mode (disable, allow, prefer, require, verify-ca, verify-full) |
| `DATABASE_MAX_CONNECTIONS` | No | `10` | Maximum connections in pool |
| `DATABASE_CONNECTION_TIMEOUT` | No | `30` | Connection timeout in seconds |
| `ENV` | No | `local` | Environment (`local`, `production`) |
| `RUST_LOG` | No | `info` | Logging level (`error`, `warn`, `info`, `debug`, `trace`) |

*Either `DATABASE_URL` OR the individual database parameters are required.

### Database Schema

The application automatically creates the following tables:

```sql
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Posts table
CREATE TABLE IF NOT EXISTS posts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    content TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id);
CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts(created_at DESC);
```

## 📊 API Documentation

### User Endpoints

#### Create User
```http
POST /api/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

**Response (201 Created):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "John Doe",
  "email": "john@example.com",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

#### Get User
```http
GET /api/users/{id}
```

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "John Doe",
  "email": "john@example.com",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

#### Update User
```http
PUT /api/users/{id}
Content-Type: application/json

{
  "name": "Jane Doe",
  "email": "jane@example.com"
}
```

#### Delete User
```http
DELETE /api/users/{id}
```

**Response:** `204 No Content`

### Post Endpoints

#### Create Post
```http
POST /api/posts
Content-Type: application/json

{
  "user_id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "My First Post",
  "content": "This is the content of my post"
}
```

#### Get Posts
```http
GET /api/posts
GET /api/posts?user_id=550e8400-e29b-41d4-a716-446655440000
```

### Error Responses

All errors return JSON in the following format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid email format"
  }
}
```

**HTTP Status Codes:**
- `200` - Success (GET, PUT)
- `201` - Created (POST)
- `204` - No Content (DELETE)
- `400` - Bad Request (validation errors)
- `404` - Not Found
- `409` - Conflict (duplicate email)
- `500` - Internal Server Error

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test models::user

# Check code formatting
cargo fmt --check

# Run clippy for linting
cargo clippy -- -D warnings
```

## 📈 Performance

- **Cold Start**: < 2 seconds on Cloud Run
- **Response Time**: < 100ms for health checks
- **Memory Usage**: ~50MB baseline
- **Concurrent Requests**: 100+ per instance

## 🔒 Security

- Input validation on all endpoints
- SQL injection prevention (PostgreSQL parameterized queries)
- Connection pooling with secure credential management
- No sensitive data in error responses
- HTTPS enforced in production
- Secrets managed via Google Secret Manager

## 📝 Logging

The application uses structured JSON logging:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "message": "Creating new user",
  "fields": {
    "email": "user@example.com",
    "request_id": "req_123"
  }
}
```

## 🚨 Monitoring

### Health Check
- Endpoint: `GET /health`
- Response: `200 OK` with body `"OK"`
- Response time: < 100ms

### Metrics
- Request count and latency
- Error rates by endpoint
- Database query performance
- Memory and CPU usage

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🆘 Troubleshooting

### Common Issues

**Database Connection Failed**
- Verify `DATABASE_URL` or individual database parameters are correct
- Check if the PostgreSQL database exists and is accessible
- Verify SSL mode is appropriate for your database (use `require` for Neon)
- Ensure network connectivity to the database server
- Check database credentials and permissions

**Connection Pool Issues**
- Monitor connection pool metrics in logs
- Adjust `DATABASE_MAX_CONNECTIONS` if needed
- Check `DATABASE_CONNECTION_TIMEOUT` settings
- Verify database server can handle the connection load

**Port Already in Use**
- Change the `PORT` environment variable
- Kill existing processes using the port: `lsof -ti:8080 | xargs kill`

**Cloud Run Deployment Failed**
- Check that all required APIs are enabled
- Verify secrets are created in Secret Manager
- Ensure the service account has proper permissions

**High Memory Usage**
- Monitor with `docker stats` or Cloud Run metrics
- Consider reducing `max-instances` if needed
- Check for memory leaks in custom code

### PostgreSQL/Neon Database Regions

Common Neon regions you can choose from:
- `us-east-1` - US East (N. Virginia)
- `us-west-2` - US West (Oregon)
- `eu-central-1` - Europe (Frankfurt)
- `ap-southeast-1` - Asia Pacific (Singapore)

### Database Connection Troubleshooting

If you encounter database connection issues:

1. **Verify Connection String Format**
   ```bash
   # Correct format for Neon:
   postgresql://username:password@ep-example-123456.us-east-1.aws.neon.tech/neondb?sslmode=require
   ```

2. **Test Connection Manually**
   ```bash
   # Using psql (if installed)
   psql "postgresql://username:password@host:port/database?sslmode=require"
   ```

3. **Check SSL Requirements**
   ```bash
   # Neon requires SSL, ensure sslmode=require in connection string
   # For local development, you might use sslmode=disable
   ```

4. **Verify Database Exists**
   ```bash
   # Check that the database name in your connection string exists
   # Default Neon database is usually 'neondb'
   ```

5. **Check Network Connectivity**
   ```bash
   # Test if you can reach the database host
   ping ep-example-123456.us-east-1.aws.neon.tech
   ```

### Getting Help

- Check the [Issues](https://github.com/your-repo/issues) page
- Review Cloud Run logs: `gcloud run logs tail word-rest-api --region=asia-northeast1`
- Enable debug logging: `RUST_LOG=debug`
- Test database connectivity: Check application startup logs for connection errors

---

Built with ❤️ using Rust, Axum, and PostgreSQL (Neon)
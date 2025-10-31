# Rust REST API with Axum + Turso + Cloud Run

A high-performance REST API built with Rust, using Axum web framework, Turso (distributed SQLite) database, and deployed on Google Cloud Run.

## 🚀 Features

- **Fast & Efficient**: Built with Rust and Axum for maximum performance
- **Distributed Database**: Turso (libSQL) for global edge replication
- **Cloud Native**: Designed for Google Cloud Run with auto-scaling
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
- **Database**: Turso (distributed SQLite)
- **Database Driver**: libsql
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
- Turso CLI installed
- Docker (for containerization)
- Google Cloud SDK (for deployment)

### 1. Database Setup (Turso)

```bash
# Install Turso CLI
curl -sSfL https://get.tur.so/install.sh | bash

# Login to Turso
turso auth login

# Create a new database
turso db create word-rest-api-db --location nrt

# Get database URL and create auth token
turso db show word-rest-api-db --url
turso db tokens create word-rest-api-db
```

### 2. Local Development

```bash
# Clone the repository
git clone <repository-url>
cd word-rest-api

# Copy environment template
cp .env.example .env

# Edit .env with your Turso credentials
# TURSO_DATABASE_URL=libsql://word-rest-api-db.turso.io
# TURSO_AUTH_TOKEN=your-auth-token

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
  -e TURSO_DATABASE_URL="your-database-url" \
  -e TURSO_AUTH_TOKEN="your-auth-token" \
  word-rest-api:latest
```

## ☁️ Cloud Run Deployment

### Prerequisites

1. Google Cloud Project with billing enabled
2. Required APIs enabled:
   - Cloud Run API
   - Artifact Registry API
   - Secret Manager API

### 1. Setup Google Cloud

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

### 2. Store Secrets

```bash
# Store Turso credentials in Secret Manager
echo -n "your-database-url" | gcloud secrets create turso-database-url --data-file=-
echo -n "your-auth-token" | gcloud secrets create turso-auth-token --data-file=-
```

### 3. Deploy to Cloud Run

```bash
# Deploy using source-based deployment
gcloud run deploy word-rest-api \
  --source . \
  --region asia-northeast1 \
  --platform managed \
  --allow-unauthenticated \
  --set-secrets="TURSO_DATABASE_URL=turso-database-url:latest,TURSO_AUTH_TOKEN=turso-auth-token:latest" \
  --set-env-vars="ENV=production" \
  --memory 512Mi \
  --cpu 1 \
  --max-instances 10 \
  --min-instances 0 \
  --timeout 300
```

### 4. Alternative: Manual Docker Deployment

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
  --set-secrets="TURSO_DATABASE_URL=turso-database-url:latest,TURSO_AUTH_TOKEN=turso-auth-token:latest" \
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
| `TURSO_DATABASE_URL` | Yes | - | Turso database connection URL (libsql://word-rest-api-db.turso.io) |
| `TURSO_AUTH_TOKEN` | Yes | - | Turso authentication token |
| `ENV` | No | `local` | Environment (`local`, `production`) |
| `RUST_LOG` | No | `info` | Logging level (`error`, `warn`, `info`, `debug`, `trace`) |

### Database Schema

The application automatically creates the following tables:

```sql
-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,           -- UUID v4
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at INTEGER NOT NULL,   -- Unix timestamp
    updated_at INTEGER NOT NULL    -- Unix timestamp
);

-- Posts table
CREATE TABLE posts (
    id TEXT PRIMARY KEY,           -- UUID v4
    user_id TEXT NOT NULL,         -- Foreign key to users.id
    title TEXT NOT NULL,
    content TEXT,                  -- Optional content
    created_at INTEGER NOT NULL,   -- Unix timestamp
    updated_at INTEGER NOT NULL,   -- Unix timestamp
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
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
  "created_at": 1698765432,
  "updated_at": 1698765432
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
  "created_at": 1698765432,
  "updated_at": 1698765432
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
- SQL injection prevention (libsql parameterized queries)
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
- Verify `TURSO_DATABASE_URL` (should be libsql://word-rest-api-db.turso.io) and `TURSO_AUTH_TOKEN` are correct
- Check if the Turso database (word-rest-api-db) exists and is accessible
- Ensure network connectivity to Turso servers

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

### Getting Help

- Check the [Issues](https://github.com/your-repo/issues) page
- Review Cloud Run logs: `gcloud run logs tail word-rest-api --region=asia-northeast1`
- Enable debug logging: `RUST_LOG=debug`

---

Built with ❤️ using Rust, Axum, and Turso
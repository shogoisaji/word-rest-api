# Production Deployment Guide

This comprehensive guide covers deploying the Rust Turso API to Google Cloud Run with production-ready configurations, monitoring, and security best practices.

## 🚀 Quick Start

For a rapid deployment, use the automated scripts:

```bash
# 1. Set up Google Cloud resources
./deploy/setup-gcp.sh

# 2. Deploy the application
./deploy/deploy.sh

# 3. Run health checks
./deploy/health-check.sh
```

## 📋 Prerequisites

### Required Tools
- **Google Cloud CLI** (gcloud) - [Install Guide](https://cloud.google.com/sdk/docs/install)
- **Docker** - [Install Guide](https://docs.docker.com/get-docker/)
- **Git** - For version control
- **curl** and **jq** - For testing and health checks

### Required Accounts & Services
- **Google Cloud Project** with billing enabled
- **Turso Account** with database created - [Turso Setup](https://turso.tech/)

### Verify Prerequisites
```bash
# Check tool versions
gcloud version
docker --version
git --version
curl --version
jq --version

# Authenticate with Google Cloud
gcloud auth login
gcloud auth application-default login
```

## 🗄️ Database Setup (Turso)

### 1. Install Turso CLI
```bash
curl -sSfL https://get.tur.so/install.sh | bash
```

### 2. Create Database
```bash
# Login to Turso
turso auth login

# Create database (choose region closest to your users)
turso db create word-rest-api-db --location nrt  # Tokyo
# or
turso db create word-rest-api-db --location iad  # Washington DC

# Get database URL
turso db show word-rest-api-db --url

# Create authentication token
turso db tokens create word-rest-api-db

# Test connection (optional)
turso db shell word-rest-api-db
```

### 3. Database Schema
The application automatically creates tables on startup. The schema includes:
- `users` table with UUID primary keys
- `posts` table with foreign key relationships
- Proper indexes for performance

## ☁️ Google Cloud Setup

### 1. Automated Setup (Recommended)
```bash
# Set your project ID
export PROJECT_ID="your-project-id"

# Run the setup script
./deploy/setup-gcp.sh
```

### 2. Manual Setup (Alternative)
```bash
# Set project
export PROJECT_ID="your-project-id"
gcloud config set project $PROJECT_ID

# Enable APIs
gcloud services enable run.googleapis.com
gcloud services enable cloudbuild.googleapis.com
gcloud services enable artifactregistry.googleapis.com
gcloud services enable secretmanager.googleapis.com

# Create Artifact Registry repository
gcloud artifacts repositories create word-rest-api \
  --repository-format=docker \
  --location=asia-northeast1

# Configure Docker authentication
gcloud auth configure-docker asia-northeast1-docker.pkg.dev
```

## 🔐 Secrets Management

### Store Turso Credentials in Secret Manager
```bash
# Store database URL
echo -n "libsql://word-rest-api-db.turso.io" | \
  gcloud secrets create turso-database-url --data-file=-

# Store auth token
echo -n "your-auth-token" | \
  gcloud secrets create turso-auth-token --data-file=-

# Verify secrets
gcloud secrets list
```

### Update Secrets (when needed)
```bash
# Update database URL
echo -n "new-database-url" | \
  gcloud secrets versions add turso-database-url --data-file=-

# Update auth token
echo -n "new-auth-token" | \
  gcloud secrets versions add turso-auth-token --data-file=-
```

## 🚢 Deployment

### 1. Automated Deployment (Recommended)
```bash
# Deploy with automated script
./deploy/deploy.sh

# The script will:
# - Build Docker image
# - Push to Artifact Registry
# - Deploy to Cloud Run
# - Run basic health checks
```

### 2. Manual Deployment
```bash
# Build and tag image
docker build -t asia-northeast1-docker.pkg.dev/$PROJECT_ID/word-rest-api/word-rest-api:latest .

# Push image
docker push asia-northeast1-docker.pkg.dev/$PROJECT_ID/word-rest-api/word-rest-api:latest

# Deploy to Cloud Run
gcloud run deploy word-rest-api \
  --image=asia-northeast1-docker.pkg.dev/$PROJECT_ID/word-rest-api/word-rest-api:latest \
  --platform=managed \
  --region=asia-northeast1 \
  --allow-unauthenticated \
  --service-account=word-rest-api-sa@$PROJECT_ID.iam.gserviceaccount.com \
  --set-secrets="TURSO_DATABASE_URL=turso-database-url:latest,TURSO_AUTH_TOKEN=turso-auth-token:latest" \
  --set-env-vars="ENV=production,RUST_LOG=info" \
  --memory=512Mi \
  --cpu=1 \
  --concurrency=100 \
  --min-instances=0 \
  --max-instances=10 \
  --timeout=300s \
  --port=8080
```

### 3. Using Cloud Build (CI/CD)
```bash
# Deploy using Cloud Build
gcloud builds submit --config cloudbuild.yaml

# Or trigger from GitHub (requires setup)
gcloud builds triggers create github \
  --repo-name=your-repo \
  --repo-owner=your-username \
  --branch-pattern="^main$" \
  --build-config=cloudbuild.yaml
```

## 🏥 Health Checks & Monitoring

### 1. Comprehensive Health Check
```bash
# Run all health checks
./deploy/health-check.sh

# Quick health check only
./deploy/health-check.sh quick

# Test API endpoints
./deploy/health-check.sh api

# Test database connectivity
./deploy/health-check.sh db

# Performance test
./deploy/health-check.sh perf
```

### 2. Manual Health Verification
```bash
# Get service URL
SERVICE_URL=$(gcloud run services describe word-rest-api \
  --region=asia-northeast1 \
  --format='value(status.url)')

# Test health endpoint
curl $SERVICE_URL/health

# Test API endpoints
curl $SERVICE_URL/api/users
curl -X POST $SERVICE_URL/api/users \
  -H "Content-Type: application/json" \
  -d '{"name": "Test User", "email": "test@example.com"}'
```

### 3. Monitoring Setup
```bash
# View logs
gcloud logs tail word-rest-api --region=asia-northeast1

# View service metrics
gcloud run services describe word-rest-api --region=asia-northeast1

# Set up alerting (optional)
gcloud alpha monitoring policies create --policy-from-file=monitoring/alert-policy.yaml
```

## 🔧 Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `ENV` | No | `local` | Environment (`local`, `production`) |
| `PORT` | No | `8080` | HTTP server port (set by Cloud Run) |
| `RUST_LOG` | No | `info` | Log level (`error`, `warn`, `info`, `debug`, `trace`) |
| `TURSO_DATABASE_URL` | Yes | - | Turso database URL (from Secret Manager) |
| `TURSO_AUTH_TOKEN` | Yes | - | Turso auth token (from Secret Manager) |

### Cloud Run Configuration

| Setting | Value | Description |
|---------|-------|-------------|
| Memory | 512Mi | Sufficient for Rust application |
| CPU | 1 vCPU | Good performance for concurrent requests |
| Concurrency | 100 | Requests per instance |
| Min Instances | 0 | Cost optimization |
| Max Instances | 10 | Auto-scaling limit |
| Timeout | 300s | Request timeout |

## 🔒 Security Best Practices

### 1. Secrets Management
- ✅ Use Google Secret Manager for sensitive data
- ✅ Never commit secrets to version control
- ✅ Rotate tokens regularly
- ✅ Use least-privilege service accounts

### 2. Container Security
- ✅ Run as non-root user
- ✅ Minimal base image (Alpine Linux)
- ✅ No unnecessary dependencies
- ✅ Health checks configured

### 3. Network Security
- ✅ HTTPS enforced by Cloud Run
- ✅ CORS properly configured
- ✅ Request timeouts prevent DoS
- ✅ Input validation on all endpoints

### 4. Application Security
- ✅ SQL injection prevention (parameterized queries)
- ✅ No sensitive data in error responses
- ✅ Structured logging without secrets
- ✅ Proper error handling

## 📊 Performance Optimization

### 1. Application Level
```toml
# Cargo.toml - Release optimizations
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### 2. Container Level
- Multi-stage Docker build
- Dependency caching
- Minimal runtime image
- Proper resource allocation

### 3. Database Level
- Proper indexing
- Connection pooling
- Query optimization
- Edge replication (Turso)

### 4. Cloud Run Level
- Appropriate memory allocation
- CPU allocation
- Concurrency settings
- Regional deployment

## 🚨 Troubleshooting

### Common Issues

#### 1. Deployment Failures
```bash
# Check build logs
gcloud builds log --region=asia-northeast1

# Check service logs
gcloud logs read --service=word-rest-api --limit=50

# Check service status
gcloud run services describe word-rest-api --region=asia-northeast1
```

#### 2. Database Connection Issues
```bash
# Test Turso connection locally
turso db shell word-rest-api-db

# Verify secrets
gcloud secrets versions access latest --secret=turso-database-url
gcloud secrets versions access latest --secret=turso-auth-token

# Check service account permissions
gcloud projects get-iam-policy $PROJECT_ID
```

#### 3. Performance Issues
```bash
# Monitor metrics
gcloud run services describe word-rest-api --region=asia-northeast1

# Check resource usage
gcloud logging read "resource.type=cloud_run_revision" --limit=50

# Run performance test
./deploy/health-check.sh perf
```

### Debug Commands
```bash
# Local container debugging
docker run -it --entrypoint /bin/sh word-rest-api

# Service logs with filtering
gcloud logs read "resource.type=cloud_run_revision AND resource.labels.service_name=word-rest-api" --limit=100

# Test specific endpoints
curl -v $SERVICE_URL/health
curl -v $SERVICE_URL/api/users
```

## 🔄 Maintenance & Updates

### 1. Rolling Updates
```bash
# Deploy new version
./deploy/deploy.sh

# Rollback if needed
./deploy/deploy.sh rollback
```

### 2. Database Migrations
```bash
# Migrations run automatically on startup
# Check logs for migration status
gcloud logs read --service=word-rest-api --filter="migration"
```

### 3. Monitoring & Alerts
```bash
# Set up monitoring dashboard
gcloud monitoring dashboards create --config-from-file=monitoring/dashboard.json

# Create alert policies
gcloud alpha monitoring policies create --policy-from-file=monitoring/alerts.yaml
```

### 4. Backup & Recovery
```bash
# Turso handles backups automatically
# Export data if needed
turso db dump word-rest-api-db > backup.sql
```

## 📈 Scaling Considerations

### Horizontal Scaling
- Cloud Run auto-scales based on traffic
- Configure max instances based on expected load
- Monitor cold start times

### Database Scaling
- Turso provides automatic scaling
- Consider read replicas for global applications
- Monitor query performance

### Cost Optimization
- Use min-instances=0 for cost savings
- Monitor usage with Cloud Billing
- Optimize container resource allocation

## 🆘 Support & Resources

### Documentation
- [Cloud Run Documentation](https://cloud.google.com/run/docs)
- [Turso Documentation](https://docs.turso.tech/)
- [Axum Documentation](https://docs.rs/axum/)

### Monitoring
- Cloud Run metrics in Google Cloud Console
- Application logs via `gcloud logs`
- Custom metrics via tracing

### Getting Help
- Check service logs first
- Run health checks
- Review this troubleshooting guide
- Check GitHub issues

---

## 📝 Deployment Checklist

Before deploying to production:

- [ ] Turso database (word-rest-api-db) created and accessible
- [ ] Google Cloud project configured
- [ ] Required APIs enabled
- [ ] Secrets stored in Secret Manager
- [ ] Service account created with proper permissions
- [ ] Docker image builds successfully
- [ ] Local testing completed
- [ ] Health checks pass
- [ ] Monitoring configured
- [ ] Backup strategy in place

After deployment:

- [ ] Health checks pass
- [ ] API endpoints respond correctly
- [ ] Database operations work
- [ ] Logs are structured and readable
- [ ] Performance meets requirements
- [ ] Security scan completed
- [ ] Documentation updated
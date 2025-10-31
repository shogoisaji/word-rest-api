use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tokio::signal;
use tracing::{error, info};

use rust_turso_api::{
    config::Config,
    db::Database,
    handlers::{
        health_check,
        posts::{create_post, get_all_posts, get_post_by_id},
        users::{create_user, delete_user, get_all_users, get_user_by_id, update_user},
    },
    middleware::{create_middleware_stack, init_tracing},
};

#[tokio::main]
async fn main() {
    // Initialize structured logging
    if let Err(e) = init_tracing() {
        eprintln!("Failed to initialize tracing: {}", e);
        std::process::exit(1);
    }

    // Load configuration from environment
    let config = match Config::from_env() {
        Ok(config) => {
            info!("Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize database connection
    let database = match Database::new(&config.database_url, &config.auth_token).await {
        Ok(db) => {
            info!("Database connection established");
            Arc::new(db)
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // Run database migrations
    if let Err(e) = database.migrate().await {
        error!("Failed to run database migrations: {}", e);
        std::process::exit(1);
    }
    info!("Database migrations completed successfully");

    // Create the Axum router with all endpoints
    let app = create_router(database);

    // Create socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Starting server on {}", addr);

    // Create the server with graceful shutdown
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("Server listening on {}", addr);
            listener
        }
        Err(e) => {
            error!("Failed to bind to address {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    // Start the server with graceful shutdown handling
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    info!("Server shutdown complete");
}

/// Create the Axum router with all endpoints and middleware
fn create_router(database: Arc<Database>) -> Router {
    Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        // User management endpoints
        .route("/api/users", post(create_user))
        .route("/api/users", get(get_all_users))
        .route("/api/users/:id", get(get_user_by_id))
        .route("/api/users/:id", put(update_user))
        .route("/api/users/:id", delete(delete_user))
        // Post management endpoints
        .route("/api/posts", post(create_post))
        .route("/api/posts", get(get_all_posts))
        .route("/api/posts/:id", get(get_post_by_id))
        // Add shared state (database connection)
        .with_state(database)
        // Apply middleware stack
        .layer(create_middleware_stack())
}

/// Graceful shutdown signal handler
/// Listens for SIGTERM and SIGINT signals
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal, initiating graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM signal, initiating graceful shutdown");
        },
    }
}
use axum::http::Method;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Creates the complete middleware stack for the application
pub fn create_middleware_stack() -> ServiceBuilder<
    tower::layer::util::Stack<
        TimeoutLayer,
        tower::layer::util::Stack<
            CorsLayer,
            tower::layer::util::Stack<
                TraceLayer<
                    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
                    DefaultMakeSpan,
                    DefaultOnRequest,
                    DefaultOnResponse,
                >,
                tower::layer::util::Identity,
            >,
        >,
    >,
> {
    ServiceBuilder::new()
        // Request/response logging with tracing
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        // CORS configuration for cross-origin requests
        .layer(create_cors_layer())
        // Request timeout handling (30 seconds)
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
}

/// Creates CORS layer configuration
fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .allow_credentials(false)
}

/// Initialize structured logging with JSON format and correlation IDs
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    // Create environment filter for log levels
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Initialize tracing subscriber with JSON formatting
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(true)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
        )
        .try_init()?;

    tracing::info!("Structured logging initialized with JSON format");
    Ok(())
}
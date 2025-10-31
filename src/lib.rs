// Library root for the Rust Turso API

pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod handlers;

// Re-export commonly used types
pub use db::Database;
pub use error::ApiError;
pub use models::{User, CreateUserRequest, UpdateUserRequest, Post, CreatePostRequest};
use std::env;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub auth_token: String,
    pub environment: Environment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Local,
    Production,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists (for local development)
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("PORT must be a valid port number")?;

        let database_url = env::var("TURSO_DATABASE_URL")
            .context("TURSO_DATABASE_URL environment variable is required")?;

        let auth_token = env::var("TURSO_AUTH_TOKEN")
            .context("TURSO_AUTH_TOKEN environment variable is required")?;

        let environment = match env::var("ENV").unwrap_or_else(|_| "local".to_string()).as_str() {
            "production" | "prod" => Environment::Production,
            _ => Environment::Local,
        };

        // Validate configuration values
        Self::validate_config(&database_url, &auth_token, port)?;

        Ok(Config {
            port,
            database_url,
            auth_token,
            environment,
        })
    }

    fn validate_config(database_url: &str, auth_token: &str, port: u16) -> Result<()> {
        // Validate database URL format
        if !database_url.starts_with("libsql://") && !database_url.starts_with("file:") && !database_url.starts_with("https://") {
            anyhow::bail!("TURSO_DATABASE_URL must start with 'libsql://', 'https://', or 'file:'");
        }

        // Validate auth token is not empty
        if auth_token.trim().is_empty() {
            anyhow::bail!("TURSO_AUTH_TOKEN cannot be empty");
        }

        // Validate port range
        if port == 0 {
            anyhow::bail!("PORT must be greater than 0");
        }

        Ok(())
    }
}

impl Environment {
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Environment::Local)
    }
}
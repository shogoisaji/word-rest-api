use std::env;
use std::time::Duration;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database: DatabaseConfig,
    pub environment: Environment,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: String,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub connection_string: Option<String>, // Support for full connection string format
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

        let database = DatabaseConfig::from_env()?;

        let environment = match env::var("ENV").unwrap_or_else(|_| "local".to_string()).as_str() {
            "production" | "prod" => Environment::Production,
            _ => Environment::Local,
        };

        // Validate configuration values
        Self::validate_config(&database, port)?;

        Ok(Config {
            port,
            database,
            environment,
        })
    }

    fn validate_config(database: &DatabaseConfig, port: u16) -> Result<()> {
        // Validate port range
        if port == 0 {
            anyhow::bail!("PORT must be greater than 0");
        }

        // Validate database configuration
        database.validate()?;

        Ok(())
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self> {
        // Try to get full connection string first
        if let Ok(connection_string) = env::var("DATABASE_URL") {
            return Self::from_connection_string(&connection_string);
        }

        // Fall back to individual parameters
        let host = env::var("DATABASE_HOST")
            .or_else(|_| env::var("NEON_HOST"))
            .unwrap_or_else(|_| "localhost".to_string());

        let port = env::var("DATABASE_PORT")
            .or_else(|_| env::var("NEON_PORT"))
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .context("DATABASE_PORT/NEON_PORT must be a valid port number")?;

        let database = env::var("DATABASE_NAME")
            .or_else(|_| env::var("NEON_DATABASE"))
            .context("DATABASE_NAME or NEON_DATABASE environment variable is required")?;

        let username = env::var("DATABASE_USERNAME")
            .or_else(|_| env::var("NEON_USERNAME"))
            .context("DATABASE_USERNAME or NEON_USERNAME environment variable is required")?;

        let password = env::var("DATABASE_PASSWORD")
            .or_else(|_| env::var("NEON_PASSWORD"))
            .context("DATABASE_PASSWORD or NEON_PASSWORD environment variable is required")?;

        let ssl_mode = env::var("DATABASE_SSL_MODE")
            .unwrap_or_else(|_| "require".to_string());

        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .context("DATABASE_MAX_CONNECTIONS must be a valid number")?;

        let connection_timeout_secs = env::var("DATABASE_CONNECTION_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .context("DATABASE_CONNECTION_TIMEOUT must be a valid number of seconds")?;

        Ok(DatabaseConfig {
            host,
            port,
            database,
            username,
            password,
            ssl_mode,
            max_connections,
            connection_timeout: Duration::from_secs(connection_timeout_secs),
            connection_string: None,
        })
    }

    pub fn from_connection_string(connection_string: &str) -> Result<Self> {
        // Parse PostgreSQL connection string format
        // postgresql://username:password@host:port/database?sslmode=require
        
        if !connection_string.starts_with("postgresql://") && !connection_string.starts_with("postgres://") {
            anyhow::bail!("DATABASE_URL must start with 'postgresql://' or 'postgres://'");
        }

        // For now, we'll store the connection string and parse individual components
        // This is a simplified implementation - in production, you might want to use a URL parsing library
        let url = connection_string.strip_prefix("postgresql://")
            .or_else(|| connection_string.strip_prefix("postgres://"))
            .unwrap();

        // Extract components (simplified parsing)
        let parts: Vec<&str> = url.split('@').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid DATABASE_URL format");
        }

        let auth_part = parts[0];
        let host_db_part = parts[1];

        let auth_parts: Vec<&str> = auth_part.split(':').collect();
        if auth_parts.len() != 2 {
            anyhow::bail!("Invalid DATABASE_URL format - missing username or password");
        }

        let username = auth_parts[0].to_string();
        let password = auth_parts[1].to_string();

        let host_db_parts: Vec<&str> = host_db_part.split('/').collect();
        if host_db_parts.len() < 2 {
            anyhow::bail!("Invalid DATABASE_URL format - missing database name");
        }

        let host_port = host_db_parts[0];
        let database_and_params = host_db_parts[1];

        let host_port_parts: Vec<&str> = host_port.split(':').collect();
        let host = host_port_parts[0].to_string();
        let port = if host_port_parts.len() > 1 {
            host_port_parts[1].parse::<u16>()
                .context("Invalid port in DATABASE_URL")?
        } else {
            5432
        };

        let database_parts: Vec<&str> = database_and_params.split('?').collect();
        let database = database_parts[0].to_string();

        // Extract SSL mode from query parameters if present
        let ssl_mode = if database_parts.len() > 1 {
            let params = database_parts[1];
            if params.contains("sslmode=") {
                params.split("sslmode=")
                    .nth(1)
                    .and_then(|s| s.split('&').next())
                    .unwrap_or("require")
                    .to_string()
            } else {
                "require".to_string()
            }
        } else {
            "require".to_string()
        };

        // Use default values for connection pool settings when using connection string
        let max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .unwrap_or(10);

        let connection_timeout_secs = env::var("DATABASE_CONNECTION_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse::<u64>()
            .unwrap_or(30);

        Ok(DatabaseConfig {
            host,
            port,
            database,
            username,
            password,
            ssl_mode,
            max_connections,
            connection_timeout: Duration::from_secs(connection_timeout_secs),
            connection_string: Some(connection_string.to_string()),
        })
    }

    pub fn validate(&self) -> Result<()> {
        // Validate host is not empty
        if self.host.trim().is_empty() {
            anyhow::bail!("Database host cannot be empty");
        }

        // Validate port range
        if self.port == 0 {
            anyhow::bail!("Database port must be greater than 0");
        }

        // Validate database name is not empty
        if self.database.trim().is_empty() {
            anyhow::bail!("Database name cannot be empty");
        }

        // Validate username is not empty
        if self.username.trim().is_empty() {
            anyhow::bail!("Database username cannot be empty");
        }

        // Validate password is not empty
        if self.password.trim().is_empty() {
            anyhow::bail!("Database password cannot be empty");
        }

        // Validate SSL mode
        match self.ssl_mode.as_str() {
            "disable" | "allow" | "prefer" | "require" | "verify-ca" | "verify-full" => {},
            _ => anyhow::bail!("Invalid SSL mode. Must be one of: disable, allow, prefer, require, verify-ca, verify-full"),
        }

        // Validate connection pool settings
        if self.max_connections == 0 {
            anyhow::bail!("Max connections must be greater than 0");
        }

        if self.connection_timeout.as_secs() == 0 {
            anyhow::bail!("Connection timeout must be greater than 0");
        }

        Ok(())
    }

    pub fn to_connection_string(&self) -> String {
        if let Some(ref conn_str) = self.connection_string {
            conn_str.clone()
        } else {
            format!(
                "postgresql://{}:{}@{}:{}/{}?sslmode={}",
                self.username, self.password, self.host, self.port, self.database, self.ssl_mode
            )
        }
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
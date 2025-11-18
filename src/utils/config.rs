// src/utils/config.rs
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub qdrant: QdrantConfig,
    pub agents: AgentConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub max_connections: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QdrantConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgentConfig {
    pub max_agents: usize,
    pub session_timeout_seconds: u64,
    pub sync_interval_seconds: u64,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("config/default").required(false))
            // Add in a local configuration file
            .add_source(File::with_name("config/local").required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                cors_origins: vec!["http://localhost:3000".to_string(), "http://localhost:3001".to_string()],
                max_connections: 100,
            },
            qdrant: QdrantConfig {
                endpoint: "http://localhost:6333".to_string(),
                api_key: None,
                timeout_seconds: 30,
            },
            agents: AgentConfig {
                max_agents: 10,
                session_timeout_seconds: 3600, // 1 hour
                sync_interval_seconds: 30,
            },
        }
    }
}

// Helper function to initialize tracing based on environment
pub fn init_tracing() {
    use tracing_subscriber::{EnvFilter};
    use tracing_subscriber::fmt;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("cerebus_rex=debug,axum=debug"));

    fmt().with_env_filter(env_filter).init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_creation() {
        // This test might fail if config files don't exist, so we'll use default
        let settings = Settings::default();
        assert_eq!(settings.server.port, 3000);
        assert_eq!(settings.qdrant.endpoint, "http://localhost:6333");
    }
}
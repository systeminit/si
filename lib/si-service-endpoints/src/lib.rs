use std::{
    net::SocketAddr,
    sync::Arc,
};

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

pub mod axum_integration;

pub mod server;

#[derive(Debug, Error)]
pub enum ServiceEndpointsError {
    #[error("HTTP server error: {0}")]
    HttpServer(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ServiceEndpointsError>;

/// Service endpoints implementation for health and config
pub struct DefaultServiceEndpoints {
    service_name: String,
    config: Arc<Value>,
}

impl DefaultServiceEndpoints {
    /// Create a new DefaultServiceEndpoints with a JSON config value
    pub fn new(service_name: impl Into<String>, config: Value) -> Self {
        Self {
            service_name: service_name.into(),
            config: Arc::new(config),
        }
    }

    /// Create from any serializable config object
    pub fn from_config<T: Serialize>(service_name: impl Into<String>, config: &T) -> Result<Self> {
        let value = serde_json::to_value(config)?;
        Ok(Self::new(service_name, value))
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn config(&self) -> &Value {
        &self.config
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceEndpointsConfig {
    pub enabled: bool,
    pub bind_address: SocketAddr,
    pub health_endpoint: String,
    pub config_endpoint: String,
}

impl Default for ServiceEndpointsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            health_endpoint: "/health".to_string(),
            config_endpoint: "/config".to_string(),
        }
    }
}

impl ServiceEndpointsConfig {
    /// Create a new config with endpoints enabled on the given port
    pub fn new(port: u16) -> Self {
        Self {
            enabled: true,
            bind_address: format!("127.0.0.1:{port}").parse().unwrap(),
            ..Default::default()
        }
    }

    /// Disable the endpoints
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

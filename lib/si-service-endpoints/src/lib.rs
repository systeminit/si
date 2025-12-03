use std::{
    net::SocketAddr,
    sync::Arc,
};

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

pub mod axum_integration;

pub mod server;

/// Sensitive field name patterns that should be redacted in config output.
/// Add any substring that appears in field names containing sensitive data.
const SENSITIVE_FIELD_PATTERNS: &[&str] = &[
    "auth",
    "credential",
    "creds",
    "key",
    "password",
    "secret",
    "token",
];

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

    /// Create from any serializable config object, redacting sensitive fields for safe exposure
    pub fn from_config<T: Serialize>(service_name: impl Into<String>, config: &T) -> Result<Self> {
        let mut value = serde_json::to_value(config)?;
        redact_sensitive_fields(&mut value);
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

/// Recursively walk a JSON value and redact fields with sensitive names
fn redact_sensitive_fields(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                let key_lower = key.to_lowercase();
                let is_sensitive = SENSITIVE_FIELD_PATTERNS
                    .iter()
                    .any(|pattern| key_lower.contains(pattern));

                if is_sensitive {
                    *val = Value::String("...".to_string());
                } else {
                    redact_sensitive_fields(val);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                redact_sensitive_fields(item);
            }
        }
        _ => {}
    }
}

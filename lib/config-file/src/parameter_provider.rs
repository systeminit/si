use std::error::Error;

use async_trait::async_trait;
use config::{
    ConfigBuilder,
    Value,
};
use serde::{
    Deserialize,
    Serialize,
};
use tracing::{
    trace,
    warn,
};

#[derive(Debug, thiserror::Error)]
pub enum ParameterError {
    #[error("Failed to fetch parameters: {0}")]
    Fetch(String),
    #[error(transparent)]
    Other(#[from] Box<dyn Error + Send + Sync>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub value: Option<String>,
    pub r#type: Option<ParameterType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    StringList,
}

#[async_trait]
pub trait ParameterProvider: Send + Sync {
    async fn get_parameters_by_path(&self, path: String) -> Result<Vec<Parameter>, ParameterError>;
    async fn environment(&self) -> String;
}

pub struct ParameterSource<P: ParameterProvider> {
    provider: P,
    service_name: String,
}

impl<P: ParameterProvider> ParameterSource<P> {
    pub fn new(provider: P, service_name: String) -> Self {
        Self {
            provider,
            service_name,
        }
    }

    /// get all global params, then all service-specific params, and merge the service-specific
    /// params over the globals
    pub async fn load<St: config::builder::BuilderState>(
        &self,
        builder: ConfigBuilder<St>,
    ) -> Result<ConfigBuilder<St>, config::ConfigError> {
        let environment = self.provider.environment().await;
        let global_path = format!("si/{environment}/global");
        let global_params = match self
            .provider
            .get_parameters_by_path(global_path.clone())
            .await
        {
            Ok(params) => params,
            Err(e) => {
                warn!(error = %e, "Failed to load global parameters");
                return Ok(builder);
            }
        };

        let service_path = format!("si/{}/{}", environment, self.service_name);
        let service_params = match self
            .provider
            .get_parameters_by_path(service_path.clone())
            .await
        {
            Err(e) => {
                warn!(error = %e, "Failed to load service parameters");
                // failures are okay here, we assume we only want to use globals
                vec![]
            }
            Ok(params) => params,
        };

        let mut builder = builder;

        for param in global_params {
            let key = extract_config_key(&global_path, &param.name);
            if !key.is_empty() {
                if let Some(value) = param.value {
                    let config_value = create_flexible_value(&value);
                    builder = builder.set_override(key, config_value)?;
                }
            }
        }

        for param in service_params {
            let key = extract_config_key(&service_path, &param.name);
            if !key.is_empty() {
                if let Some(value) = param.value {
                    let config_value = create_flexible_value(&value);
                    builder = builder.set_override(key, config_value)?;
                }
            }
        }

        Ok(builder)
    }
}

/// Extract the config key from a parameter name
/// e.g., "/si/tools/global/pg/hostname" -> "pg.hostname"
fn extract_config_key(prefix: &str, name: &str) -> String {
    // Normalize both the prefix and name to ensure they have consistent formats
    let norm_prefix = prefix.trim_start_matches('/');
    let norm_name = name.trim_start_matches('/');

    if let Some(stripped) = norm_name.strip_prefix(norm_prefix) {
        stripped.trim_start_matches('/').replace('/', ".")
    } else {
        norm_name.replace('/', ".")
    }
}

fn create_flexible_value(value: &str) -> Value {
    if value.contains(',') {
        trace!("Treating value '{}' as a comma-separated list", value);
        return Value::from(
            value
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>(),
        );
    }

    if let Ok(int_val) = value.parse::<i64>() {
        trace!("Parsed '{}' as integer", value);
        return Value::from(int_val);
    }

    if let Ok(float_val) = value.parse::<f64>() {
        trace!("Parsed '{}' as float", value);
        return Value::from(float_val);
    }

    match value.to_lowercase().as_str() {
        "true" => {
            trace!("Parsed '{}' as boolean true", value);
            return Value::from(true);
        }
        "false" => {
            trace!("Parsed '{}' as boolean false", value);
            return Value::from(false);
        }
        _ => {}
    }

    trace!("Treating '{}' as string", value);
    Value::from(value.to_string())
}

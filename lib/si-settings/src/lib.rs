use std::{
    env,
    fmt::Debug,
};

use async_trait::async_trait;
pub use config_file::{
    ConfigMap,
    ValueKind,
    parameter_provider::ParameterProvider,
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SettingsError {
    #[error(transparent)]
    ConfigFile(#[from] config_file::ConfigFileError),
}

pub type Result<T> = std::result::Result<T, SettingsError>;

pub trait StandardConfig: Sized {
    type Builder: Default;

    /// Constructs a builder for creating a Config
    #[must_use]
    fn builder() -> Self::Builder {
        Self::Builder::default()
    }
}

#[async_trait]
pub trait StandardConfigFile:
    Clone + Debug + Default + DeserializeOwned + Send + Serialize + Sized + Sync + 'static
{
    type Error: From<SettingsError>;

    fn layered_load<F>(
        app_name: impl AsRef<str>,
        set_func: F,
    ) -> std::result::Result<Self, Self::Error>
    where
        F: FnOnce(&mut ConfigMap),
    {
        let app_name = app_name.as_ref();
        config_file::layered_load(
            app_name,
            "toml",
            &Some(format!("SI_{}_CONFIG", app_name.to_uppercase())),
            &Some(format!("SI_{}", app_name.to_uppercase())),
            set_func,
        )
        .map_err(SettingsError::ConfigFile)
        .map_err(Into::into)
    }

    async fn layered_load_with_provider<F, P>(
        app_name: impl AsRef<str> + Send,
        parameter_provider: Option<(P, String)>,
        set_func: F,
    ) -> std::result::Result<Self, Self::Error>
    where
        F: FnOnce(&mut ConfigMap) + Send,
        P: ParameterProvider + 'static,
    {
        let app_name = app_name.as_ref();
        config_file::layered_load_with_provider(
            app_name,
            "toml",
            &Some(format!("SI_{}_CONFIG", app_name.to_uppercase())),
            &Some(format!("SI_{}", app_name.to_uppercase())),
            parameter_provider,
            set_func,
        )
        .await
        .map_err(SettingsError::ConfigFile)
        .map_err(Into::into)
    }
}

#[allow(clippy::disallowed_methods)]
pub fn get_host_environment() -> String {
    env::var("SI_HOSTENV").unwrap_or_else(|_| "local".to_string())
}

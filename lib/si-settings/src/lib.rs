use std::fmt::Debug;

use config_file::ConfigMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

mod canonical_file;

pub use canonical_file::{CanonicalFile, CanonicalFileError};

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
        let p = config_file::layered_load(
            app_name,
            "toml",
            &Some(format!("SI_{}_CONFIG", app_name.to_uppercase())),
            &Some(format!("SI_{}_", app_name.to_uppercase())),
            set_func,
        )
        .map_err(SettingsError::ConfigFile)
        .map_err(Into::into);
        p
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
struct Veritech {
    pub ws_url: String,
    pub http_url: String,
}

impl Default for Veritech {
    fn default() -> Self {
        Self {
            ws_url: "ws://localhost:5157".to_string(),
            http_url: "http://localhost:5157".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
struct Service {
    pub port: u16,
}

impl Default for Service {
    fn default() -> Self {
        Self { port: 5156 }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
struct Paging {
    pub key: sodiumoxide::crypto::secretbox::Key,
}

impl Default for Paging {
    fn default() -> Self {
        Paging {
            key: sodiumoxide::crypto::secretbox::gen_key(),
        }
    }
}

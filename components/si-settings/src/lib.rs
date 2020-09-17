use config;
use config::{Config, Environment, File as ConfigFile};
use serde::Deserialize;
use sodiumoxide;
use tracing::{event, Level};

use std::env;

pub mod error;

use crate::error::{Result, SettingsError};

#[derive(Debug, Deserialize, Clone)]
pub struct Db {
    pub cluster_url: String,
    pub cluster_user: String,
    pub cluster_password: String,
    pub bucket_name: String,
    pub scan_consistency: String,
}

impl Default for Db {
    fn default() -> Self {
        Db {
            cluster_url: String::from("couchbase://127.0.0.1"),
            cluster_user: String::from("si"),
            cluster_password: String::from("bugbear"),
            bucket_name: String::from("si"),
            scan_consistency: String::from("NotBounded"),
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Service {
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Paging {
    pub key: sodiumoxide::crypto::secretbox::Key,
}

impl Default for Paging {
    fn default() -> Self {
        Paging {
            key: sodiumoxide::crypto::secretbox::gen_key(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtEncrypt {
    pub key: sodiumoxide::crypto::secretbox::Key,
}

impl Default for JwtEncrypt {
    fn default() -> Self {
        JwtEncrypt {
            key: sodiumoxide::crypto::secretbox::gen_key(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Vernemq {
    pub server_uri: Option<String>,
    pub shared_topic_id: Option<String>,
}

impl Default for Vernemq {
    fn default() -> Self {
        Vernemq {
            server_uri: Some(String::from("tcp://localhost:1883")),
            shared_topic_id: None,
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Settings {
    pub db: Db,
    pub service: Service,
    pub paging: Paging,
    pub jwt_encrypt: JwtEncrypt,
    pub vernemq: Option<Vernemq>,
}

impl Settings {
    pub fn vernemq_server_uri(&self) -> String {
        match self.vernemq {
            Some(ref vernemq) => match vernemq.server_uri {
                Some(ref server_uri) => server_uri.to_string(),
                None => Vernemq::default().server_uri.unwrap(),
            },
            None => Vernemq::default().server_uri.unwrap(),
        }
    }

    pub fn vernemq_shared_topic_id(&self) -> Result<&str> {
        let key = "vernemq.shared_topic_id";

        match self.vernemq {
            Some(ref vernemq) => match vernemq.shared_topic_id {
                Some(ref shared_topic_id) => Ok(shared_topic_id),
                None => Err(SettingsError::Required(key)),
            },
            None => Err(SettingsError::Required(key)),
        }
    }

    #[tracing::instrument]
    pub fn new() -> Result<Settings> {
        if let Err(()) = sodiumoxide::init() {
            return Err(SettingsError::SodiumOxideInit);
        }

        let mut s = Config::default();

        // Start off by merging in the "default" configuration file
        event!(Level::DEBUG, "Loading config/default.toml");
        s.merge(ConfigFile::with_name("config/default"))?;
        event!(Level::DEBUG, ?s, "Loaded config/default.toml");

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        let env = env::var("RUN_ENV").unwrap_or("development".into());
        event!(
            Level::DEBUG,
            ?env,
            "Loading environment configuration (config/env.toml)"
        );
        s.merge(ConfigFile::with_name(&format!("config/{}", env)).required(false))?;
        event!(Level::DEBUG, ?s, ?env, "Loaded config/env.toml");

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        event!(Level::DEBUG, "Loading config/local.toml");
        s.merge(ConfigFile::with_name("config/local").required(false))?;
        event!(Level::DEBUG, ?s, "Loaded config/local.toml");

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `SI_DEBUG=1 ./target/app` would set the `debug` key
        event!(Level::DEBUG, "Loading SI_* environment");
        s.merge(Environment::with_prefix("SI").separator("__"))?;
        event!(Level::DEBUG, ?s, "Loaded SI_* environment");

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into().map_err(SettingsError::ConfigError)
    }
}

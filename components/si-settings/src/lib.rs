use config;
use config::{Config, Environment, File as ConfigFile};
use serde::Deserialize;
use sodiumoxide;
use std::path::PathBuf;
use tracing::{event, Level};

use std::{cmp, env};

pub mod error;

use crate::error::{Result, SettingsError};

const MAX_POOL_SIZE_MINIMUM: usize = 32;

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Pg {
    pub user: String,
    pub password: String,
    pub dbname: String,
    pub application_name: String,
    pub hostname: String,
    pub port: u16,
    pub pool_max_size: usize,
    pub pool_timeout_wait_secs: Option<u64>,
    pub pool_timeout_create_secs: Option<u64>,
    pub pool_timeout_recycle_secs: Option<u64>,
}

impl Default for Pg {
    fn default() -> Self {
        let pool_max_size = cmp::max(MAX_POOL_SIZE_MINIMUM, num_cpus::get_physical() * 4);

        Pg {
            user: String::from("si"),
            password: String::from("bugbear"),
            dbname: String::from("si"),
            application_name: String::from("si-sdf"),
            hostname: String::from("localhost"),
            port: 5432,
            pool_max_size,
            pool_timeout_wait_secs: None,
            pool_timeout_create_secs: None,
            pool_timeout_recycle_secs: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Nats {
    pub url: String,
}

impl Default for Nats {
    fn default() -> Self {
        Nats {
            url: "localhost".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Veritech {
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
pub struct EventLogFs {
    pub root: PathBuf,
}

impl Default for EventLogFs {
    fn default() -> Self {
        Self {
            root: PathBuf::from("/tmp/si-sdf-event-log-fs"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct Service {
    pub port: u16,
}

impl Default for Service {
    fn default() -> Self {
        Self { port: 5156 }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
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
#[serde(default)]
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

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Settings {
    pub pg: Pg,
    pub nats: Nats,
    pub veritech: Veritech,
    pub event_log_fs: EventLogFs,
    pub service: Service,
    pub paging: Paging,
    pub jwt_encrypt: JwtEncrypt,
}

impl Settings {
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

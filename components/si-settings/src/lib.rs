use config;
use config::{Config, Environment, File as ConfigFile};
use serde::Deserialize;
use sodiumoxide;
use tracing::{event, Level};

use std::env;

pub mod error;

use crate::error::{Result, SettingsError};

#[derive(Debug, Deserialize)]
pub struct Db {
    pub cluster_url: String,
    pub cluster_user: String,
    pub cluster_password: String,
    pub bucket_name: String,
    pub scan_consistency: String,
}

#[derive(Debug, Deserialize)]
pub struct Service {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Paging {
    pub key: sodiumoxide::crypto::secretbox::Key,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub db: Db,
    pub service: Service,
    pub paging: Paging,
}

impl Settings {
    #[tracing::instrument]
    pub fn new() -> Result<Settings> {
        if let Err(()) = sodiumoxide::init() {
            return Err(SettingsError::SodiumOxideInit);
        }

        let mut s = Config::new();

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

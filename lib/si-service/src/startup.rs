//! Service/server start pre-processing for establishing/reigstering service:
//! - Health [to come]
//! - Version
//! - Anything else to do async or sync during service startup

use std::{
    env,
    io,
    path::Component,
};

use glob::glob;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    fs::File,
    io::AsyncReadExt,
};

/// An error that can be returned when starting the process for the binary
#[derive(Debug, Error)]
pub enum StartupError {
    /// When the version could not be established
    #[error("Failed to establish version: {0}")]
    Signal(#[source] io::Error),
}

/// Gracefully start a service and conduct pre-processing of service handler
pub async fn startup(service: &str) -> Result<(), std::io::Error> {
    let executable_path = match env::current_exe() {
        Ok(exe_path) => exe_path,
        Err(_) => {
            info!(
                "could not establish running executable path for {}",
                service
            );
            return Ok(());
        }
    };

    let executable_path = match executable_path.canonicalize() {
        Ok(exe_path) => exe_path,
        Err(_) => {
            info!("could not canonicalize executable path for {}", service);
            return Ok(());
        }
    };

    // Check if it's a dev build (i.e. running from buck-out)
    if executable_path
        .components()
        .any(|path| Component::Normal("buck-out".as_ref()) == path)
    {
        debug!(
            "development build (buck) detected for {}, no metadata can be reported",
            service
        );
        return Ok(());
    }

    let metadata_candidates = match glob(&format!("/etc/nix-omnibus/{service}/*/metadata.json")) {
        Ok(iter) => iter,
        Err(_) => {
            info!("metadata candidates could not be found for {}", service);
            return Ok(());
        }
    };

    let mut metadata_candidates = match metadata_candidates.collect::<Result<Vec<_>, _>>() {
        Ok(vec) => vec,
        Err(_) => {
            info!(
                "could not collect PathBufs from metadata_candidates for {}",
                service
            );
            return Ok(());
        }
    };

    // Sort them lexically so that the latest (if there is more than one) is at the bottom
    // There is a minor issue here if we `downgrade` a single running server/host there is the potential
    // that this reports the newer version, rather than the rollback version due to how we are
    // lexically sorting. At the time of writing there was no viable method to determine which exact
    // metadata file should be referenced.
    metadata_candidates.sort();

    // Take the last one (the latest)
    let metadata_file_path = match metadata_candidates.pop() {
        Some(file_path) => file_path,
        None => {
            info!(
                "could not read appropriate metadata files for {} to determine version",
                service
            );
            return Ok(());
        }
    };

    // Read contents of metadata file
    let mut metadata_file_handler = match File::open(&metadata_file_path).await {
        Ok(metadata_file_handler) => metadata_file_handler,
        Err(_) => {
            info!("metadata file could not be read for {}", service);
            return Ok(());
        }
    };

    let mut file_contents = String::new();
    metadata_file_handler
        .read_to_string(&mut file_contents)
        .await?;

    let metadata_file_path_str = metadata_file_path.as_path().display().to_string();

    info!(file_contents, metadata_file_path_str, "metadata contents:");

    Ok(())
}

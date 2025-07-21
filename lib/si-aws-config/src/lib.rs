//! This crate provides a wrapper for aws config

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::{
    fmt::Debug,
    result,
};

use aws_config::{
    SdkConfig,
    meta::region::RegionProviderChain,
    retry::RetryConfig,
};
use aws_sdk_sts::error::SdkError;
use telemetry::prelude::*;
use thiserror::Error;

const DEFAULT_MAX_ATTEMPTS: u32 = 10;
const DEFAULT_FALLBACK_REGION: &str = "us-east-1";

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum AwsConfigError {
    #[error("AWS STS error: {0}")]
    Sts(String),
}

impl AwsConfigError {
    fn from_sdk_error<T: Debug>(error: SdkError<T>) -> Self {
        AwsConfigError::Sts(format!("{error:?}"))
    }
}

type Result<T> = result::Result<T, AwsConfigError>;

/// Wrapper for aws config
#[derive(Debug, Clone)]
pub struct AwsConfig {}

impl AwsConfig {
    /// Get an aws config from the environment with built-in retry and sane defaults. It will
    /// attempt to verify the credentials by getting the established caller identity.
    pub async fn from_env() -> Result<SdkConfig> {
        let retry_config = RetryConfig::adaptive().with_max_attempts(DEFAULT_MAX_ATTEMPTS);

        let config = aws_config::from_env()
            .retry_config(retry_config)
            .region(RegionProviderChain::default_provider().or_else(DEFAULT_FALLBACK_REGION))
            .load()
            .await;

        let sts = aws_sdk_sts::Client::new(&config);
        let ident = sts
            .get_caller_identity()
            .send()
            .await
            .map_err(AwsConfigError::from_sdk_error)?;

        let Some(account) = ident.account() else {
            return Err(AwsConfigError::Sts(
                "Unable to get account from credentials".to_string(),
            ));
        };

        info!("Successfully validated AWS Credentials for account: {account}");

        Ok(config)
    }
}

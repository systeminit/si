//! This crate provides a client for interacting with AWS SSM, Parameter Store in particular

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

use std::fmt::Debug;

use aws_sdk_ssm::{
    config::{
        Credentials,
        Region,
    },
    error::SdkError,
    types::{
        Parameter,
        ParameterType,
    },
};
use si_aws_config::{
    AwsConfig,
    AwsConfigError,
};
use telemetry::prelude::*;
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ParameterStoreClientError {
    #[error("AWS Config Error error: {0}")]
    AwsConfig(#[from] AwsConfigError),
    #[error("AWS Parameter Store error: {0}")]
    AwsParameterStore(String),
    #[error("Parameter path invalid, must start with /: {0}")]
    InvalidPath(String),
    #[error("Parameter not found: {0}")]
    ParameterNotFound(String),
    #[error("Parameter path not found: {0}")]
    PathNotFound(String),
}

impl ParameterStoreClientError {
    fn from_sdk_error<T: Debug>(error: SdkError<T>) -> Self {
        ParameterStoreClientError::AwsParameterStore(format!("{error:?}"))
    }
}

type ParameterStoreClientResult<T> = Result<T, ParameterStoreClientError>;

/// A client for communicating with ssm.
#[derive(Debug, Clone)]
pub struct ParameterStoreClient {
    inner: Box<aws_sdk_ssm::Client>,
}

impl ParameterStoreClient {
    /// Creates a new [client for interacting with SSM Parameter Store](ParameterStoreClient).
    #[instrument(name = "parameter_store_client.new", level = "info")]
    pub async fn new() -> ParameterStoreClientResult<Self> {
        let config = AwsConfig::from_env().await?;
        let client = aws_sdk_ssm::Client::new(&config);
        Ok(Self {
            inner: Box::new(client),
        })
    }

    /// Creates a [ParameterStoreClient] configured for testing (e.g., LocalStack).
    pub fn new_for_test(endpoint: String) -> Self {
        let shared_config = aws_sdk_ssm::config::Builder::new()
            .region(Region::new("us-east-1"))
            .endpoint_url(endpoint)
            .credentials_provider(Credentials::new("test", "test", None, None, "test"))
            .behavior_version_latest()
            .build();

        let client = aws_sdk_ssm::Client::from_conf(shared_config);

        Self {
            inner: Box::new(client),
        }
    }

    /// Create a String type parameter
    pub async fn create_string_parameter(
        &self,
        name: String,
        value: String,
    ) -> ParameterStoreClientResult<()> {
        self.create_parameter(name, value, ParameterType::String)
            .await
    }

    #[instrument(name = "parameter_store_client.create_parameter", level = "debug")]
    async fn create_parameter(
        &self,
        name: String,
        value: String,
        parameter_type: ParameterType,
    ) -> ParameterStoreClientResult<()> {
        self.inner
            .put_parameter()
            .name(name.clone())
            .value(value)
            .r#type(parameter_type)
            .overwrite(true)
            .send()
            .await
            .map_err(ParameterStoreClientError::from_sdk_error)?;

        Ok(())
    }

    /// Gets a specific parameter by name
    #[instrument(name = "parameter_store_client.parameter", level = "debug")]
    pub async fn get_parameter(&self, name: String) -> ParameterStoreClientResult<Parameter> {
        let result = self
            .inner
            .get_parameter()
            .name(name.clone())
            .send()
            .await
            .map_err(ParameterStoreClientError::from_sdk_error)?;

        result
            .parameter()
            .cloned()
            .ok_or(ParameterStoreClientError::ParameterNotFound(name))
    }

    /// Gets all parameters under a path, e.g. /si/global/pg
    #[instrument(name = "parameter_store_client.parameters", level = "debug")]
    pub async fn parameters_by_path(
        &self,
        path: String,
    ) -> ParameterStoreClientResult<Vec<Parameter>> {
        if !path.starts_with("/") {
            return Err(ParameterStoreClientError::InvalidPath(path));
        }

        let mut parameters: Vec<Parameter> = Vec::new();
        let mut next_token = None;

        loop {
            let mut request = self
                .inner
                .get_parameters_by_path()
                .path(path.clone())
                .recursive(true)
                .with_decryption(true);

            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let result = request
                .send()
                .await
                .map_err(ParameterStoreClientError::from_sdk_error)?;

            next_token = result.next_token().map(|token| token.to_string());

            parameters.extend(result.parameters().to_vec());

            if next_token.is_none() {
                break;
            }
        }

        if parameters.is_empty() {
            return Err(ParameterStoreClientError::PathNotFound(path));
        }

        Ok(parameters)
    }
}

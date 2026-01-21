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

use telemetry::prelude::*;
use thiserror::Error;

use crate::Mode;

pub(crate) mod env;
pub(crate) mod ssm;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ParameterStoreError {
    #[error("attempted write in env mode (write operations not supported in this mode)")]
    AttemptedWriteInEnvMode,
    #[error("parameter path invalid (must start with '/'): {0}")]
    InvalidPath(String),
    #[error("parameter not found: {0}")]
    ParameterNotFound(String),
    #[error("parameter path not found: {0}")]
    PathNotFound(String),
    #[error("ssm parameter store client error: {0}")]
    SsmParameterStoreClient(#[from] si_data_ssm::ParameterStoreClientError),
}

pub(crate) type ParameterStoreResult<T> = Result<T, ParameterStoreError>;

#[remain::sorted]
#[derive(Debug, Clone)]
pub enum ParameterStore {
    Env(env::EnvParameterStorage),
    Ssm(si_data_ssm::ParameterStoreClient),
}

impl ParameterStore {
    pub async fn new(mode: Mode, test_endpoint: Option<String>) -> ParameterStoreResult<Self> {
        match mode {
            Mode::Env => Ok(Self::Env(env::EnvParameterStorage::new())),
            Mode::Ssm => {
                let ssm_client = if let Some(endpoint) = test_endpoint {
                    si_data_ssm::ParameterStoreClient::new_for_test(endpoint)
                } else {
                    si_data_ssm::ParameterStoreClient::new().await?
                };
                Ok(Self::Ssm(ssm_client))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterType {
    String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub value: String,
    pub parameter_type: ParameterType,
}

impl Parameter {
    pub fn new(name: String, value: String, parameter_type: ParameterType) -> Self {
        Self {
            name,
            value,
            parameter_type,
        }
    }
}

impl From<Parameter> for innit_core::Parameter {
    fn from(p: Parameter) -> Self {
        Self {
            name: p.name,
            value: Some(p.value),
            r#type: Some(match p.parameter_type {
                ParameterType::String => innit_core::ParameterType::String,
            }),
        }
    }
}

#[async_trait::async_trait]
pub trait ParameterStoreKind: std::fmt::Debug + Send + Sync {
    async fn get_parameter(&self, name: String) -> ParameterStoreResult<Parameter>;
    async fn parameters_by_path(&self, path: String) -> ParameterStoreResult<Vec<Parameter>>;
    async fn create_string_parameter(
        &self,
        name: String,
        value: String,
    ) -> ParameterStoreResult<()>;
}

#[async_trait::async_trait]
impl ParameterStoreKind for ParameterStore {
    async fn get_parameter(&self, name: String) -> ParameterStoreResult<Parameter> {
        match self {
            Self::Env(storage) => storage.get_parameter(name).await,
            Self::Ssm(client) => {
                <si_data_ssm::ParameterStoreClient as ParameterStoreKind>::get_parameter(
                    client, name,
                )
                .await
            }
        }
    }

    async fn parameters_by_path(&self, path: String) -> ParameterStoreResult<Vec<Parameter>> {
        match self {
            Self::Env(storage) => storage.parameters_by_path(path).await,
            Self::Ssm(client) => {
                <si_data_ssm::ParameterStoreClient as ParameterStoreKind>::parameters_by_path(
                    client, path,
                )
                .await
            }
        }
    }

    async fn create_string_parameter(
        &self,
        name: String,
        value: String,
    ) -> ParameterStoreResult<()> {
        match self {
            Self::Env(storage) => storage.create_string_parameter(name, value).await,
            Self::Ssm(client) => {
                <si_data_ssm::ParameterStoreClient as ParameterStoreKind>::create_string_parameter(
                    client, name, value,
                )
                .await
            }
        }
    }
}

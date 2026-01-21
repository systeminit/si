use axum::extract::FromRef;
use tokio_util::sync::CancellationToken;

use crate::{
    parameter_cache::ParameterCache,
    parameter_storage::ParameterStore,
};

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub parameter_cache: ParameterCache,
    pub parameter_storage: ParameterStore,
    shutdown_token: CancellationToken,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parameter_cache: ParameterCache,
        parameter_storage: ParameterStore,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            parameter_cache,
            parameter_storage,
            shutdown_token,
        }
    }
}

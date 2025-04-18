use axum::extract::FromRef;
use si_data_ssm::ParameterStoreClient;
use tokio_util::sync::CancellationToken;

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub parameter_store_client: ParameterStoreClient,
    shutdown_token: CancellationToken,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parameter_store_client: ParameterStoreClient,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            parameter_store_client,
            shutdown_token,
        }
    }
}

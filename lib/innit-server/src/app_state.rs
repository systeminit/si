use aws_sdk_ssm::Client;
use axum::extract::FromRef;
use tokio_util::sync::CancellationToken;

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub ssm_client: Client,
    shutdown_token: CancellationToken,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(ssm_client: Client, shutdown_token: CancellationToken) -> Self {
        Self {
            ssm_client,
            shutdown_token,
        }
    }
}

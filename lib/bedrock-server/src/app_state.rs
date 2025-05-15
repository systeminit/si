use axum::extract::FromRef;
use tokio_util::sync::CancellationToken;
use si_data_nats::NatsClient;
use std::sync::Arc;

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub nats: Arc<NatsClient>,
    shutdown_token: CancellationToken,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        nats: Arc<NatsClient>,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            nats,
            shutdown_token,
        }
    }
}

use std::sync::Arc;

use axum::extract::FromRef;
use si_data_nats::NatsClient;
use tokio_util::sync::CancellationToken;
use bedrock_core::{ArtifactStoreConfig};

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub nats: Arc<NatsClient>,
    pub artifact_config: ArtifactStoreConfig,
    shutdown_token: CancellationToken,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(nats: Arc<NatsClient>, artifact_config: ArtifactStoreConfig, shutdown_token: CancellationToken) -> Self {
        Self {
            nats,
            artifact_config,
            shutdown_token,
        }
    }
}

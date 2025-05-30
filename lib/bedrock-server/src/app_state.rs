use std::sync::Arc;

use axum::extract::FromRef;
use bedrock_core::ArtifactStoreConfig;
use s3::creds::Credentials;
use si_data_nats::NatsClient;
use tokio_util::sync::CancellationToken;

#[remain::sorted]
#[derive(Debug, Eq, PartialEq)]
pub enum ShutdownSource {}

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub nats: Arc<NatsClient>,
    pub artifact_config: ArtifactStoreConfig,
    pub aws_credentials: Credentials,
    shutdown_token: CancellationToken,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        nats: Arc<NatsClient>,
        artifact_config: ArtifactStoreConfig,
        aws_credentials: Credentials,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            nats,
            artifact_config,
            aws_credentials,
            shutdown_token,
        }
    }
}

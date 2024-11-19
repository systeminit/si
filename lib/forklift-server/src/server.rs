use std::{fmt, future::Future, io, sync::Arc};

use si_data_nats::{jetstream, NatsClient};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;
use tokio_util::sync::CancellationToken;

use crate::config::Config;

mod app;

pub(crate) use app::AppSetupError;

const DURABLE_CONSUMER_NAME: &str = "forklift-server";

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("app setup error: {0}")]
    AppSetup(#[from] AppSetupError),
    #[error("join error: {0}")]
    Join(#[from] JoinError),
    #[error("naxum error: {0}")]
    Naxum(#[source] io::Error),
    #[error("si data nats error: {0}")]
    SiDataNats(#[from] si_data_nats::Error),
}

type Result<T> = std::result::Result<T, ServerError>;

/// Server metadata, used with telemetry.
#[derive(Clone, Debug)]
pub(crate) struct ServerMetadata {
    #[allow(dead_code)]
    instance_id: String,
    #[allow(dead_code)]
    job_invoked_provider: &'static str,
}

impl ServerMetadata {
    /// Returns the server's unique instance id.
    #[allow(dead_code)]
    pub(crate) fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Returns the job invoked provider.
    #[allow(dead_code)]
    pub(crate) fn job_invoked_provider(&self) -> &str {
        self.job_invoked_provider
    }
}

/// The forklift server instance with its inner naxum task.
pub struct Server {
    metadata: Arc<ServerMetadata>,
    shutdown_token: CancellationToken,
    // TODO(nick): remove option once this is working.
    inner_audit_logs: Option<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>>,
    inner_billing_events: Box<dyn Future<Output = io::Result<()>> + Unpin + Send>,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("metadata", &self.metadata)
            .field("shutdown_token", &self.shutdown_token)
            .finish()
    }
}

impl Server {
    /// Creates a forklift server with a running naxum task.
    #[instrument(name = "forklift.init.from_config", level = "info", skip_all)]
    pub async fn from_config(config: Config, token: CancellationToken) -> Result<Self> {
        let metadata = Arc::new(ServerMetadata {
            instance_id: config.instance_id().into(),
            job_invoked_provider: "si",
        });

        let nats = Self::connect_to_nats(&config).await?;
        let connection_metadata = nats.metadata_clone();
        let jetstream_context = jetstream::new(nats);

        let inner_audit_logs = if config.enable_audit_logs_app() {
            Some(
                app::audit_logs(
                    jetstream_context.clone(),
                    DURABLE_CONSUMER_NAME.to_string(),
                    connection_metadata.clone(),
                    config.concurrency_limit(),
                    config.audit(),
                    token.clone(),
                )
                .await?,
            )
        } else {
            None
        };
        let inner_billing_events = app::billing_events(
            jetstream_context,
            DURABLE_CONSUMER_NAME.to_string(),
            connection_metadata,
            config.concurrency_limit(),
            config.data_warehouse_stream_name(),
            token.clone(),
        )
        .await?;

        Ok(Self {
            metadata,
            inner_audit_logs,
            inner_billing_events,
            shutdown_token: token,
        })
    }

    /// Infallible wrapper around running the inner naxum task(s).
    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(error = ?err, "error while running forklift main loop");
        }
    }

    /// Fallibly awaits the inner naxum task(s).
    pub async fn try_run(self) -> Result<()> {
        match self.inner_audit_logs {
            Some(inner_audit_logs) => {
                info!("running two apps: audit logs and billing events");
                let (inner_audit_logs_result, inner_billing_events_result) = futures::join!(
                    tokio::spawn(inner_audit_logs),
                    tokio::spawn(self.inner_billing_events)
                );
                inner_audit_logs_result?.map_err(ServerError::Naxum)?;
                inner_billing_events_result?.map_err(ServerError::Naxum)?;
            }
            None => {
                info!("running one app: billing events");
                self.inner_billing_events
                    .await
                    .map_err(ServerError::Naxum)?;
            }
        }
        info!("forklift main loop shutdown complete");
        Ok(())
    }

    #[instrument(name = "forklift.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(config: &Config) -> Result<NatsClient> {
        let client = NatsClient::new(config.nats()).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }
}

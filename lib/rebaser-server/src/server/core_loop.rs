use dal::{
    DalContext, DalContextBuilder, DalLayerDb, JobQueueProcessor, ServicesContext, Tenancy,
    TransactionsError, Visibility, WorkspacePk,
};
use futures::FutureExt;
use futures::StreamExt;
use si_crypto::SymmetricCryptoService;
use si_data_nats::jetstream::{AckKind, JetstreamError};
use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use si_layer_cache::activities::rebase::RebaseStatus;
use si_layer_cache::activities::AckRebaseRequest;
use si_layer_cache::activities::RebaserRequestsWorkQueueStream;
use si_layer_cache::LayerDbError;
use std::sync::Arc;
use std::time::Instant;
use stream_cancel::StreamExt as CancelStreamExt;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::watch;

use crate::server::rebase::perform_rebase;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CoreLoopSetupError {
    #[error("jetstream error: {0}")]
    Jetstream(#[from] JetstreamError),
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("serde json erorr: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
}

impl From<TransactionsError> for CoreLoopSetupError {
    fn from(e: TransactionsError) -> Self {
        Self::Transactions(Box::new(e))
    }
}

type CoreLoopSetupResult<T> = Result<T, CoreLoopSetupError>;

#[allow(clippy::too_many_arguments)]
pub(crate) async fn setup_and_run_core_loop(
    pg_pool: PgPool,
    nats: NatsClient,
    veritech: veritech_client::Client,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    symmetric_crypto_service: SymmetricCryptoService,
    encryption_key: Arc<veritech_client::CycloneEncryptionKey>,
    shutdown_watch_rx: watch::Receiver<()>,
    layer_db: DalLayerDb,
) -> CoreLoopSetupResult<()> {
    let services_context = ServicesContext::new(
        pg_pool,
        nats.clone(),
        job_processor,
        veritech.clone(),
        encryption_key,
        None,
        None,
        symmetric_crypto_service,
        layer_db.clone(),
    );

    info!("getting dal context builder");
    let ctx_builder = DalContext::builder(services_context.clone(), false);

    info!("subscribing to work queue");
    let stream = layer_db.activity().rebase().subscribe_work_queue().await?;

    info!("setup complete, entering core loop");
    core_loop_infallible(ctx_builder, stream, shutdown_watch_rx).await;
    info!("exited core loop");

    Ok(())
}

async fn core_loop_infallible(
    ctx_builder: DalContextBuilder,
    stream: RebaserRequestsWorkQueueStream,
    mut shutdown_watch_rx: watch::Receiver<()>,
) {
    let mut stream = stream.take_until_if(Box::pin(shutdown_watch_rx.changed().map(|_| true)));

    while let Some(unprocessed_message) = stream.next().await {
        let message = match unprocessed_message {
            Ok(processed) => processed,
            Err(err) => {
                error!(error = ?err, "error when pull message off stream");
                continue;
            }
        };

        if let Err(err) = message.ack_with(AckKind::Progress).await {
            error!(error = ?err, "could not ack with progress, going to continue anyway");
        }

        let ctx_builder = ctx_builder.clone();
        tokio::spawn(async move {
            perform_rebase_and_reply_infallible(ctx_builder, &message).await;
            if let Err(err) = message.ack_with(AckKind::Ack).await {
                error!(?message, ?err, "failing acking message");
            }
        });
    }
}

async fn perform_rebase_and_reply_infallible(
    ctx_builder: DalContextBuilder,
    message: &AckRebaseRequest,
) {
    let start = Instant::now();

    let mut ctx = match ctx_builder.build_default().await {
        Ok(ctx) => ctx,
        Err(err) => {
            error!(error = ?err, "unable to build dal context");
            return;
        }
    };
    ctx.update_visibility_deprecated(Visibility::new_head());
    ctx.update_tenancy(Tenancy::new(WorkspacePk::NONE));

    let rebase_status = perform_rebase(&mut ctx, message)
        .await
        .unwrap_or_else(|err| {
            error!(error = ?err, ?message, "performing rebase failed, attempting to reply");
            RebaseStatus::Error {
                message: err.to_string(),
            }
        });

    if let Err(e) = message.ack().await {
        error!(error = ?e, ?message, "failed to acknolwedge the nats message after rebase; likely a timeout");
    }

    if let Err(e) = ctx
        .layer_db()
        .activity()
        .rebase()
        .finished(
            rebase_status,
            message.payload.to_rebase_change_set_id,
            message.payload.onto_workspace_snapshot_address,
            message.metadata.clone(),
            message.id,
        )
        .await
    {
        error!(error = ?e, ?message, "failed to send rebase finished activity");
    }

    info!(
        ?message,
        "perform rebase and reply total: {:?}",
        start.elapsed()
    );
}

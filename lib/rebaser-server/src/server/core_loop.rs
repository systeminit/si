use dal::{
    DalContext, DalContextBuilder, DalLayerDb, JobQueueProcessor, ServicesContext, Tenancy,
    TransactionsError, Visibility, WorkspacePk, WsEvent,
};

use si_crypto::SymmetricCryptoService;
use si_data_nats::jetstream::JetstreamError;
use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use si_layer_cache::activities::rebase::RebaseStatus;
use si_layer_cache::activities::ActivityRebaseRequest;
use si_layer_cache::LayerDbError;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio_util::task::TaskTracker;
use ulid::Ulid;

use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

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
    shutdown_watch_rx: oneshot::Receiver<()>,
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
    run_me(ctx_builder, stream, shutdown_watch_rx).await;
    info!("exited core loop");

    Ok(())
}

async fn run_me(
    ctx_builder: DalContextBuilder,
    rebase_activity_channel: UnboundedReceiver<ActivityRebaseRequest>,
    shutdown_watch_rx: oneshot::Receiver<()>,
) {
    let tracker = TaskTracker::new();

    let change_set_distributor_tracker = tracker.clone();
    let distributor_handle = tracker.spawn(async move {
        change_set_distributor(
            ctx_builder,
            change_set_distributor_tracker,
            rebase_activity_channel,
        )
        .await
    });
    let watch_handle = tokio::spawn(async move {
        match shutdown_watch_rx.await {
            Ok(_) => info!("shutting down core loop from signal"),
            Err(error) => info!(?error, "shutdown sender exited, shutting down"),
        }
    });
    tokio::select! {
        _ = distributor_handle => {
           info!("rebase change set distributor has exited");
        },
        _ = watch_handle => {
            tracker.close();
           info!("graceful shutdown");
        }
    }
}

// Hey, you. Yes, you. The one trying to figure out: "hey, self, why is it that the rebaser has so
// many tokio tasks and is running out of memory?". Well, here is why. If you actually try and use
// it in the current state, you're going to get a worker per change set id, which then serializes
// all the work. That's great, because it's how the rebaser is supposed to work. It's bad, because
// we never garbage collect them.
//
// Fletcher is in the middle of making this work permanently, but for now - this will work. If it's
// getting weird in resource utilization, you can always reboot the rebaser. :)
async fn change_set_distributor(
    ctx_builder: DalContextBuilder,
    tracker: TaskTracker,
    mut rebase_activity_channel: UnboundedReceiver<ActivityRebaseRequest>,
) -> CoreLoopSetupResult<()> {
    let mut change_set_channels: HashMap<Ulid, UnboundedSender<ActivityRebaseRequest>> =
        HashMap::new();
    while let Some(message) = rebase_activity_channel.recv().await {
        let rebase_change_set_id = message.payload.to_rebase_change_set_id;
        if change_set_channels.contains_key(&message.payload.to_rebase_change_set_id) {
            if let Some(channel) = change_set_channels.get(&message.payload.to_rebase_change_set_id)
            {
                if let Err(_error) = channel.send(message) {
                    info!(
                        "Worker for {} has closed its channel; removing worker",
                        &rebase_change_set_id
                    );
                    change_set_channels.remove(&rebase_change_set_id);
                }
            }
        } else {
            let (tx, rx) = unbounded_channel();
            if let Err(error) = tx.send(message) {
                error!(
                    ?error,
                    "Couldn't publish to a change set channel we just created; bug!"
                );
            }
            change_set_channels.insert(rebase_change_set_id, tx);
            let change_set_ctx_builder = ctx_builder.clone();
            tracker.spawn(async move { core_loop_change_set(change_set_ctx_builder, rx).await });
        }
    }
    Ok(())
}

async fn core_loop_change_set(
    ctx_builder: DalContextBuilder,
    mut rebase_activity_channel: UnboundedReceiver<ActivityRebaseRequest>,
) {
    while let Some(message) = rebase_activity_channel.recv().await {
        let ctx_builder = ctx_builder.clone();
        perform_rebase_and_reply_infallible(ctx_builder, &message).await;
    }
}

async fn perform_rebase_and_reply_infallible(
    ctx_builder: DalContextBuilder,
    message: &ActivityRebaseRequest,
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

    match WsEvent::change_set_written(&ctx, message.payload.to_rebase_change_set_id.into()).await {
        Ok(mut event) => {
            event.set_workspace_pk(message.metadata.tenancy.workspace_pk.into_inner().into());
            if let Err(error) = event.publish_immediately(&ctx).await {
                error!(?error, "failed to send wsevent for change set updated");
            }
            warn!(?event, "for real, we did publish it");
        }
        Err(error) => {
            error!(
                ?error,
                "failed to construct a wsevent; this really shouldn't be happening. bug!"
            );
        }
    }

    info!(
        ?message,
        "perform rebase and reply total: {:?}",
        start.elapsed()
    );
}

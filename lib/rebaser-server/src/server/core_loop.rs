use dal::{
    DalContext, DalContextBuilder, JobQueueProcessor, ServicesContext, Tenancy, TransactionsError,
    Visibility, WorkspacePk,
};
use futures::FutureExt;
use futures::StreamExt;
use rebaser_core::{
    RebaserMessagingConfig, ReplyRebaseMessage, RequestRebaseMessage, SubjectGenerator,
};
use si_crypto::SymmetricCryptoService;
use si_data_nats::jetstream::{AckKind, JetstreamError, Stream, REPLY_SUBJECT_HEADER_NAME};
use si_data_nats::subject::ToSubject;
use si_data_nats::NatsClient;
use si_data_pg::PgPool;
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
    messaging_config: RebaserMessagingConfig,
    content_store_pg_pool: PgPool,
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
        messaging_config.clone(),
        content_store_pg_pool,
    );

    // Setup the subjects.
    let subject_all = SubjectGenerator::all(messaging_config.subject_prefix());
    let subject_root = SubjectGenerator::root(messaging_config.subject_prefix());
    info!(%subject_all, %subject_root, "created services context and prepared subjects");

    // Setup the stream and the consumer.
    let jetstream_ctx = nats.clone().to_jetstream_ctx();
    info!(%subject_all, %subject_root, "finding or creating stream");
    let rebaser_jetstream_stream = jetstream_ctx
        .get_or_create_work_queue_stream(&subject_root, vec![subject_all.clone()])
        .await?;

    info!(%subject_all, %subject_root, "finding or creating durable consumer");
    let consumer = jetstream_ctx
        .get_or_create_durable_consumer(&rebaser_jetstream_stream, &subject_root)
        .await?;

    info!(%subject_all, %subject_root, "getting stream from consumer");
    let stream = consumer.stream().await?;

    info!("getting dal context builder");
    let ctx_builder = DalContext::builder(services_context.clone(), false);

    info!("setup complete, entering core loop");
    core_loop_infallible(ctx_builder, stream, shutdown_watch_rx).await;
    info!("exited core loop");

    Ok(())
}

async fn core_loop_infallible(
    ctx_builder: DalContextBuilder,
    stream: Stream,
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

        // Deserialize the message payload so that we can process it.
        let request_message: RequestRebaseMessage =
            match serde_json::from_slice(message.message.payload.to_vec().as_slice()) {
                Ok(deserialized) => deserialized,
                Err(err) => {
                    error!(error = ?err, ?message, "failed to deserialize message payload");
                    continue;
                }
            };

        // Pull the reply subject off of the message.
        let reply_subject = if let Some(headers) = &message.headers {
            if let Some(value) = headers.get(REPLY_SUBJECT_HEADER_NAME.clone()) {
                value.to_string()
            } else {
                // NOTE(nick): we may actually want to process the message anyway, but things would be super messed up
                // at that point... because no one should be sending messages exterior to rebaser clients.
                error!(
                    ?message,
                    "no reply subject found in headers, skipping messages because we cannot reply"
                );
                continue;
            }
        } else {
            // NOTE(nick): we may actually want to process the message anyway, but things would be super messed up
            // at that point... because no one should be sending messages exterior to rebaser clients.
            error!(
                ?message,
                "no headers found, skipping message because we cannot reply"
            );
            continue;
        };

        tokio::spawn(perform_rebase_and_reply_infallible(
            ctx_builder.clone(),
            request_message,
            reply_subject,
        ));
    }
}

async fn perform_rebase_and_reply_infallible(
    ctx_builder: DalContextBuilder,
    message: RequestRebaseMessage,
    reply_subject: impl ToSubject,
) {
    let start = Instant::now();

    let mut ctx = match ctx_builder.build_default().await {
        Ok(ctx) => ctx,
        Err(err) => {
            error!(error = ?err, "unable to build dal context");
            return;
        }
    };
    ctx.update_visibility(Visibility::new_head(false));
    ctx.update_tenancy(Tenancy::new(WorkspacePk::NONE));

    let reply_subject = reply_subject.to_subject();

    let reply_message = perform_rebase(&mut ctx, message).await.unwrap_or_else(|err| {
        error!(error = ?err, ?message, ?reply_subject, "performing rebase failed, attempting to reply");
        ReplyRebaseMessage::Error {
            message: err.to_string(),
        }
    });

    match serde_json::to_vec(&reply_message) {
        Ok(serialized_payload) => {
            if let Err(publish_err) = ctx
                .nats_conn()
                .publish(reply_subject.clone(), serialized_payload.into())
                .await
            {
                error!(error = ?publish_err, %reply_subject, "replying to requester failed");
            }
        }
        Err(serialization_err) => {
            error!(error = ?serialization_err, %reply_subject, "failed to serialize reply message");
        }
    }
    info!("perform rebase and reply total: {:?}", start.elapsed());
}

use std::{
    sync::Arc,
    time::Duration,
};

use dal::DalContextBuilder;
use edda_client::EddaClient;
use nats_dead_letter_queue::DeadLetterQueue;
use pinga_client::PingaClient;
use si_data_nats::{
    NatsClient,
    async_nats::jetstream,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use crate::{
    Features,
    ServerMetadata,
};

/// Application state.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) metadata: Arc<ServerMetadata>,
    pub(crate) nats: NatsClient,
    pub(crate) pinga: PingaClient,
    pub(crate) edda: EddaClient,
    pub(crate) requests_stream: jetstream::stream::Stream,
    pub(crate) dead_letter_queue: DeadLetterQueue,
    pub(crate) ctx_builder: DalContextBuilder,
    pub(crate) quiescent_period: Duration,
    pub(crate) token: CancellationToken,
    pub(crate) server_tracker: TaskTracker,
    pub(crate) features: Features,
}

impl AppState {
    /// Creates a new [`AppState`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        pinga: PingaClient,
        edda: EddaClient,
        requests_stream: jetstream::stream::Stream,
        dead_letter_queue: DeadLetterQueue,
        ctx_builder: DalContextBuilder,
        quiescent_period: Duration,
        token: CancellationToken,
        server_tracker: TaskTracker,
        features: Features,
    ) -> Self {
        Self {
            metadata,
            nats,
            pinga,
            edda,
            requests_stream,
            dead_letter_queue,
            ctx_builder,
            quiescent_period,
            token,
            server_tracker,
            features,
        }
    }
}

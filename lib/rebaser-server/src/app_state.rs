use std::{sync::Arc, time::Duration};

use dal::DalContextBuilder;
use si_data_nats::{async_nats::jetstream, NatsClient};
use tokio_util::sync::CancellationToken;

use crate::ServerMetadata;

/// Application state.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) metadata: Arc<ServerMetadata>,
    pub(crate) nats: NatsClient,
    pub(crate) requests_stream: jetstream::stream::Stream,
    pub(crate) ctx_builder: DalContextBuilder,
    pub(crate) quiescent_period: Duration,
    pub(crate) token: CancellationToken,
}

impl AppState {
    /// Creates a new [`AppState`].
    pub(crate) fn new(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        requests_stream: jetstream::stream::Stream,
        ctx_builder: DalContextBuilder,
        quiescent_period: Duration,
        token: CancellationToken,
    ) -> Self {
        Self {
            metadata,
            nats,
            requests_stream,
            ctx_builder,
            quiescent_period,
            token,
        }
    }
}

use std::{sync::Arc, time::Duration};

use dal::DalContextBuilder;
use frigg::FriggStore;
use si_data_nats::{NatsClient, async_nats::jetstream};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::ServerMetadata;

/// Application state.
#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) metadata: Arc<ServerMetadata>,
    pub(crate) nats: NatsClient,
    pub(crate) frigg: FriggStore,
    pub(crate) requests_stream: jetstream::stream::Stream,
    pub(crate) ctx_builder: DalContextBuilder,
    pub(crate) quiescent_period: Duration,
    pub(crate) token: CancellationToken,
    pub(crate) server_tracker: TaskTracker,
}

impl AppState {
    /// Creates a new [`AppState`].
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        metadata: Arc<ServerMetadata>,
        nats: NatsClient,
        frigg: FriggStore,
        requests_stream: jetstream::stream::Stream,
        ctx_builder: DalContextBuilder,
        quiescent_period: Duration,
        token: CancellationToken,
        server_tracker: TaskTracker,
    ) -> Self {
        Self {
            metadata,
            nats,
            frigg,
            requests_stream,
            ctx_builder,
            quiescent_period,
            token,
            server_tracker,
        }
    }
}

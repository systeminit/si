use std::sync::Arc;

use dal::DalContextBuilder;
use si_data_nats::NatsClient;

use crate::server::ServerMetadata;

/// Application state.
#[derive(Clone, Debug)]
pub struct AppState {
    pub metadata: Arc<ServerMetadata>,
    pub concurrency_limit: usize,
    /// NATS client
    pub(crate) nats: NatsClient,
    /// DAL context builder for each processing request
    pub ctx_builder: DalContextBuilder,
}

impl AppState {
    /// Creates a new [`AppState`].
    pub fn new(
        metadata: Arc<ServerMetadata>,
        concurrency_limit: usize,
        nats: NatsClient,
        ctx_builder: DalContextBuilder,
    ) -> Self {
        Self {
            metadata,
            concurrency_limit,
            nats,
            ctx_builder,
        }
    }
}

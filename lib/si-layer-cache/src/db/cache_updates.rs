use std::{str::FromStr, sync::Arc};

use futures::StreamExt;
use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::{
    async_nats::jetstream::{
        self,
        consumer::{AckPolicy, DeliverPolicy},
    },
    NatsClient,
};
use strum::{AsRefStr, EnumString};
use telemetry::prelude::*;
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use crate::{
    chunking_nats::{self, ChunkedMessagesStream, ChunkingNats},
    error::LayerDbResult,
    event::LayeredEvent,
    layer_cache::LayerCache,
    nats::{self, NATS_HEADER_DB_NAME, NATS_HEADER_INSTANCE_ID, NATS_HEADER_KEY},
    LayerDbError,
};

#[remain::sorted]
#[derive(Copy, Clone, Debug, EnumString, AsRefStr)]
#[strum(serialize_all = "snake_case")]
enum CacheName {
    Cas,
    EncryptedSecret,
    NodeWeights,
    WorkspaceSnapshots,
}

pub struct CacheUpdatesTask<CasValue, EncryptedSecretValue, WorkspaceSnapshotValue, NodeWeightValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    NodeWeightValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    instance_id: Ulid,
    messages: ChunkedMessagesStream,
    cas_cache: LayerCache<Arc<CasValue>>,
    encrypted_secret_cache: LayerCache<Arc<EncryptedSecretValue>>,
    snapshot_cache: LayerCache<Arc<WorkspaceSnapshotValue>>,
    node_weight_cache: LayerCache<Arc<NodeWeightValue>>,
}

impl<CasValue, EncryptedSecretValue, WorkspaceSnapshotValue, NodeWeightValue>
    CacheUpdatesTask<CasValue, EncryptedSecretValue, WorkspaceSnapshotValue, NodeWeightValue>
where
    CasValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    EncryptedSecretValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    WorkspaceSnapshotValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    NodeWeightValue: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    const NAME: &'static str = "LayerDB::CacheUpdatesTask";

    pub async fn create(
        instance_id: Ulid,
        nats_client: &NatsClient,
        cas_cache: LayerCache<Arc<CasValue>>,
        encrypted_secret_cache: LayerCache<Arc<EncryptedSecretValue>>,
        snapshot_cache: LayerCache<Arc<WorkspaceSnapshotValue>>,
        node_weight_cache: LayerCache<Arc<NodeWeightValue>>,
        shutdown_token: CancellationToken,
    ) -> LayerDbResult<Self> {
        let context = jetstream::new(nats_client.as_inner().clone());

        let messages_stream =
            nats::layerdb_events_stream(&context, nats_client.metadata().subject_prefix())
                .await?
                .create_consumer(Self::consumer_config(instance_id))
                .await?
                .messages()
                .await?;
        let messages = ChunkingNats::chunking_messages(messages_stream, shutdown_token);

        Ok(Self {
            instance_id,
            messages,
            cas_cache,
            encrypted_secret_cache,
            snapshot_cache,
            node_weight_cache,
        })
    }

    pub async fn run(mut self) {
        while let Some(result) = self.messages.next().await {
            match result {
                Ok(msg) => {
                    if let Err(e) = msg.ack().await {
                        warn!(error = ?e, "error acknowledging message from stream");
                    }
                    let cache_update_task = CacheUpdateTask::new(
                        self.instance_id,
                        self.cas_cache.clone(),
                        self.encrypted_secret_cache.clone(),
                        self.snapshot_cache.clone(),
                        self.node_weight_cache.clone(),
                    );

                    tokio::task::spawn(async move { cache_update_task.run(msg).await });
                }
                // An error while pulling a new message
                Err(err) => {
                    warn!(error = ?err, "error receiving layerdb message");
                }
            }
        }

        debug!(task = Self::NAME, "shutdown complete");
    }

    #[inline]
    fn consumer_config(instance_id: Ulid) -> jetstream::consumer::pull::Config {
        let name = format!("cache-updates-{instance_id}");
        let description = format!("cache updates for [{name}]");

        jetstream::consumer::pull::Config {
            name: Some(name),
            description: Some(description),
            deliver_policy: DeliverPolicy::New,
            ack_policy: AckPolicy::None,
            ..Default::default()
        }
    }
}

struct CacheUpdateTask<Q, R, S, T>
where
    Q: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    R: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    S: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    instance_id: Ulid,
    cas_cache: LayerCache<Arc<Q>>,
    encrypted_secret_cache: LayerCache<Arc<R>>,
    snapshot_cache: LayerCache<Arc<S>>,
    node_weight_cache: LayerCache<Arc<T>>,
}

impl<Q, R, S, T> CacheUpdateTask<Q, R, S, T>
where
    Q: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    R: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    S: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn new(
        instance_id: Ulid,
        cas_cache: LayerCache<Arc<Q>>,
        encrypted_secret_cache: LayerCache<Arc<R>>,
        snapshot_cache: LayerCache<Arc<S>>,
        node_weight_cache: LayerCache<Arc<T>>,
    ) -> CacheUpdateTask<Q, R, S, T> {
        CacheUpdateTask {
            instance_id,
            cas_cache,
            encrypted_secret_cache,
            snapshot_cache,
            node_weight_cache,
        }
    }

    async fn process_message(&self, msg: chunking_nats::Message) -> LayerDbResult<()> {
        let instance_id = self.instance_id.to_string();
        match &msg.headers {
            Some(headers) => {
                match (
                    headers.get(NATS_HEADER_INSTANCE_ID).map(|val| val.as_str()),
                    headers.get(NATS_HEADER_DB_NAME).map(|val| val.as_str()),
                    headers.get(NATS_HEADER_KEY).map(|val| val.as_str()),
                ) {
                    // Message with expected headers
                    (Some(instance_id_str), Some(db_name_str), Some(key)) => {
                        if instance_id_str == instance_id.as_str() {
                            trace!("message received with our instance id; skipping");
                            return Ok(());
                        }

                        let cache_name = CacheName::from_str(db_name_str)
                            .map_err(|_| LayerDbError::InvalidCacheName(db_name_str.to_string()))?;

                        match cache_name {
                            CacheName::Cas => {
                                if !self.cas_cache.contains(key) {
                                    let event: LayeredEvent = postcard::from_bytes(&msg.payload)?;
                                    let memory_value = self
                                        .cas_cache
                                        .deserialize_memory_value(&event.payload.value)?;
                                    let serialized_value = Arc::try_unwrap(event.payload.value)
                                        .unwrap_or_else(|arc| (*arc).clone());
                                    self.cas_cache
                                        .insert_from_cache_updates(
                                            key.into(),
                                            memory_value,
                                            serialized_value,
                                        )
                                        .await?;
                                }
                            }
                            CacheName::EncryptedSecret => {
                                if !self.encrypted_secret_cache.contains(key) {
                                    let event: LayeredEvent = postcard::from_bytes(&msg.payload)?;
                                    let memory_value = self
                                        .encrypted_secret_cache
                                        .deserialize_memory_value(&event.payload.value)?;
                                    let serialized_value = Arc::try_unwrap(event.payload.value)
                                        .unwrap_or_else(|arc| (*arc).clone());
                                    self.encrypted_secret_cache
                                        .insert_from_cache_updates(
                                            key.into(),
                                            memory_value,
                                            serialized_value,
                                        )
                                        .await?;
                                }
                            }
                            CacheName::WorkspaceSnapshots => {
                                if !self.snapshot_cache.contains(key) {
                                    let event: LayeredEvent = postcard::from_bytes(&msg.payload)?;
                                    let memory_value = self
                                        .snapshot_cache
                                        .deserialize_memory_value(&event.payload.value)?;
                                    let serialized_value = Arc::try_unwrap(event.payload.value)
                                        .unwrap_or_else(|arc| (*arc).clone());
                                    self.snapshot_cache
                                        .insert_from_cache_updates(
                                            key.into(),
                                            memory_value,
                                            serialized_value,
                                        )
                                        .await?;
                                }
                            }
                            CacheName::NodeWeights => {
                                if !self.node_weight_cache.contains(key) {
                                    let event: LayeredEvent = postcard::from_bytes(&msg.payload)?;
                                    let memory_value = self
                                        .node_weight_cache
                                        .deserialize_memory_value(&event.payload.value)?;
                                    let serialized_value = Arc::try_unwrap(event.payload.value)
                                        .unwrap_or_else(|arc| (*arc).clone());
                                    self.node_weight_cache
                                        .insert_from_cache_updates(
                                            key.into(),
                                            memory_value,
                                            serialized_value,
                                        )
                                        .await?;
                                }
                            }
                        }
                    }
                    // Message headers are incomplete
                    _ => {
                        warn!(
                            subject = msg.subject.as_str(),
                            ?headers,
                            "message received with incomplete headers"
                        );
                        return Err(LayerDbError::CacheUpdateBadHeaders(format!(
                            "{:?}",
                            headers
                        )));
                    }
                }
            }
            None => {
                // TODO: maybe the log level isn't correct--we don't yet know if this
                // is expected or not
                warn!(
                    subject = msg.subject.as_str(),
                    "message received with no headers"
                );
                return Err(LayerDbError::CacheUpdateNoHeaders);
            }
        }
        Ok(())
    }

    async fn run(&self, msg: chunking_nats::Message) {
        match self.process_message(msg).await {
            Ok(()) => {}
            Err(e) => {
                error!(error = %e, "error processing layerdb cache update message");
            }
        }
    }
}

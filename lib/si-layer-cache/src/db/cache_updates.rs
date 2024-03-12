use std::{str::FromStr, sync::Arc};

use futures::StreamExt;
use serde::{de::DeserializeOwned, Serialize};
use si_data_nats::{async_nats::jetstream, NatsClient};
use strum::{AsRefStr, EnumString};
use telemetry::prelude::*;
use ulid::Ulid;

use crate::{
    chunking_nats::{self, ChunkedMessagesStream, ChunkingNats},
    error::LayerDbResult,
    event::LayeredEvent,
    layer_cache::LayerCache,
    nats::{self, NATS_HEADER_DB_NAME, NATS_HEADER_INSTANCE_ID, NATS_HEADER_KEY},
    LayerDbError,
};

#[derive(Copy, Clone, Debug, EnumString, AsRefStr)]
enum CacheName {
    Cas,
}

pub struct CacheUpdates<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    instance_id: Ulid,
    messages: ChunkedMessagesStream,
    cas_cache: LayerCache<Arc<V>>,
}

impl<V> CacheUpdates<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub async fn create(
        instance_id: Ulid,
        nats_client: &NatsClient,
        cas_cache: LayerCache<Arc<V>>,
    ) -> LayerDbResult<Self> {
        let context = jetstream::new(nats_client.as_inner().clone());

        let messages_stream =
            nats::layerdb_events_stream(&context, nats_client.metadata().subject_prefix())
                .await?
                .create_consumer(Self::consumer_config(instance_id))
                .await?
                .messages()
                .await?;
        let messages = ChunkingNats::chunking_messages(messages_stream);

        Ok(Self {
            instance_id,
            messages,
            cas_cache,
        })
    }

    pub async fn run(&mut self) {
        while let Some(result) = self.messages.next().await {
            match result {
                Ok(msg) => {
                    let cache_update_task =
                        CacheUpdateTask::new(self.instance_id, self.cas_cache.clone());
                    // Turns out I think it's probably dangerous to do this spawned, since we want
                    // to make sure we insert things into the cache in the order we receive them.
                    // If we spawn, we could do more at once, but at the cost of being uncertain
                    // about order.
                    //
                    // If we need to do it async, we can just spawn. But I think this is what we
                    // want.
                    cache_update_task.run(msg).await;
                }
                // An error while pulling a new message
                Err(err) => {
                    warn!(error = ?err, "error receiving layerdb message");
                }
            }
        }
    }

    #[inline]
    fn consumer_config(instance_id: Ulid) -> jetstream::consumer::pull::Config {
        let name = format!("cache-updates-{instance_id}");
        let description = format!("cache updates for [{name}]");

        jetstream::consumer::pull::Config {
            name: Some(name),
            description: Some(description),
            ..Default::default()
        }
    }
}

struct CacheUpdateTask<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    instance_id: Ulid,
    cas_cache: LayerCache<Arc<V>>,
}

impl<V> CacheUpdateTask<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn new(instance_id: Ulid, cas_cache: LayerCache<Arc<V>>) -> CacheUpdateTask<V> {
        CacheUpdateTask {
            instance_id,
            cas_cache,
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

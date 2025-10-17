use std::{
    collections::{
        HashMap,
        hash_map::Entry,
    },
    sync::Arc,
    time::Duration,
};

use bytes::{
    BufMut,
    Bytes,
    BytesMut,
};
use chrono::prelude::*;
use futures::StreamExt;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_nats::{
    HeaderMap,
    NatsClient,
    async_nats::jetstream::{
        self,
        Message,
        consumer::{
            DeliverPolicy,
            pull::Stream,
        },
    },
    jetstream::context::Context,
};
use si_events::{
    Actor,
    Tenancy,
    WebEvent,
};
use strum::AsRefStr;
use telemetry::tracing::{
    debug,
    warn,
};
use tokio::{
    sync::mpsc::{
        UnboundedReceiver,
        UnboundedSender,
    },
    task::JoinHandle,
};
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};
use ulid::Ulid;

use crate::{
    LayerDbError,
    db::serialize,
    error::LayerDbResult,
    nats::{
        self,
        NATS_HEADER_DB_NAME,
        NATS_HEADER_INSTANCE_ID,
        NATS_HEADER_KEY,
        subject,
    },
};

const DEFAULT_CHUNK_SIZE: usize = 128 * 1024;
const MAX_BYTES: i64 = 1024 * 1024; // mirrors settings in Synadia NATs

const HEADER_EVENT_ID: &str = "X-EVENT-ID";
const HEADER_CHECKSUM: &str = "X-CHECKSUM";
const HEADER_SIZE: &str = "X-SIZE";
const HEADER_NUM_CHUNKS: &str = "X-NUM-CHUNKS";
const HEADER_CUR_CHUNK: &str = "X-CUR-CHUNK";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LayeredEventMetadata {
    pub tenancy: Tenancy,
    pub actor: Actor,
    pub timestamp: DateTime<Utc>,
}

impl LayeredEventMetadata {
    pub fn new(tenancy: Tenancy, actor: Actor) -> Self {
        LayeredEventMetadata {
            tenancy,
            actor,
            timestamp: Utc::now(),
        }
    }
}

pub use si_id::LayeredEventId;

#[remain::sorted]
#[derive(AsRefStr, Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LayeredEventKind {
    CasInsertion,
    ChangeBatchEvict,
    ChangeBatchWrite,
    EncryptedSecretInsertion,
    FuncRunLogWrite,
    FuncRunWrite,
    Raw,
    RebaseBatchEvict,
    RebaseBatchWrite,
    SnapshotEvict,
    SnapshotWrite,
    SplitRebaseBatchEvict,
    SplitRebaseBatchWrite,
    SplitSnapshotSubGraphEvict,
    SplitSnapshotSubGraphWrite,
    SplitSnapshotSuperGraphEvict,
    SplitSnapshotSuperGraphWrite,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayeredEventPayload {
    pub db_name: Arc<String>,
    pub key: Arc<str>,
    pub sort_key: Arc<String>,
    pub value: Arc<Vec<u8>>,
}

impl LayeredEventPayload {
    pub fn new(
        db_name: Arc<String>,
        key: Arc<str>,
        value: Arc<Vec<u8>>,
        sort_key: Arc<String>,
    ) -> LayeredEventPayload {
        LayeredEventPayload {
            db_name,
            key,
            value,
            sort_key,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LayeredEvent {
    pub event_id: LayeredEventId,
    pub event_kind: LayeredEventKind,
    pub key: Arc<str>,
    pub metadata: LayeredEventMetadata,
    pub payload: LayeredEventPayload,
    pub web_events: Option<Vec<WebEvent>>,
}

impl LayeredEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        event_kind: LayeredEventKind,
        db_name: Arc<String>,
        key: Arc<str>,
        value: Arc<Vec<u8>>,
        sort_key: Arc<String>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> Self {
        LayeredEvent {
            event_id: LayeredEventId::new(),
            event_kind,
            key: key.clone(),
            metadata: LayeredEventMetadata::new(tenancy, actor),
            payload: LayeredEventPayload::new(db_name, key, value, sort_key),
            web_events,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayeredEventClient {
    context: Context,
    prefix: Option<Arc<str>>,
    instance_id: Ulid,
}

impl LayeredEventClient {
    pub fn new(prefix: Option<String>, instance_id: Ulid, context: Context) -> Self {
        Self {
            prefix: prefix.map(|s| s.into()),
            context,
            instance_id,
        }
    }

    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    // Publishes messages in chunks if they need it!
    pub async fn publish(
        &self,
        event: Arc<LayeredEvent>,
    ) -> LayerDbResult<JoinHandle<LayerDbResult<()>>> {
        let prefix = self.prefix.clone();
        let instance_id = self.instance_id;
        let context = self.context.clone();
        let join_handle = tokio::spawn(async move {
            let prefix = prefix.as_deref();
            let subject = subject::for_event(prefix, &event);
            let mut headers = HeaderMap::new();
            headers.insert(NATS_HEADER_DB_NAME, event.payload.db_name.as_str());
            headers.insert(NATS_HEADER_KEY, event.payload.key.as_ref());
            headers.insert(NATS_HEADER_INSTANCE_ID, instance_id.to_string().as_str());
            let (payload, _) = serialize::to_vec(&event)?;

            let event_id = Ulid::new();
            let object_size = payload.len();
            let chunk_size = DEFAULT_CHUNK_SIZE;
            let checksum = blake3::hash(&payload);
            let num_chunks = object_size.div_ceil(chunk_size);

            headers.insert(HEADER_EVENT_ID, event_id.to_string().as_str());
            headers.insert(HEADER_SIZE, object_size.to_string().as_str());
            headers.insert(HEADER_CHECKSUM, checksum.to_string().as_str());
            headers.insert(HEADER_NUM_CHUNKS, num_chunks.to_string().as_str());

            let mut cur_chunk = 1;

            for chunk in payload.chunks(chunk_size) {
                let mut headers = headers.clone();
                headers.insert(HEADER_CUR_CHUNK, cur_chunk.to_string().as_str());

                context
                    .publish_with_headers(subject.clone(), headers, Bytes::copy_from_slice(chunk))
                    .await?
                    .await?;

                cur_chunk += 1;
            }
            Ok(())
        });
        Ok(join_handle)
    }
}

pub struct LayeredEventServer {
    nats_client: NatsClient,
    tracker: TaskTracker,
    shutdown_token: CancellationToken,
    instance_id: Ulid,
    tx: UnboundedSender<LayeredEvent>,
    buffers: HashMap<String, BytesMut>,
}

impl LayeredEventServer {
    const NAME: &'static str = "LayerDB::LayeredEventServer";

    pub fn create(
        instance_id: Ulid,
        nats_client: NatsClient,
        shutdown_token: CancellationToken,
    ) -> (Self, UnboundedReceiver<LayeredEvent>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (
            Self {
                nats_client,
                shutdown_token,
                tracker: TaskTracker::new(),
                instance_id,
                tx,
                buffers: HashMap::new(),
            },
            rx,
        )
    }

    pub async fn run(&mut self) -> LayerDbResult<()> {
        let shutdown_token = self.shutdown_token.clone();

        let context = si_data_nats::jetstream::new(self.nats_client.clone());

        let mut messages_stream =
            nats::layerdb_events_stream(&context, self.nats_client.metadata().subject_prefix())
                .await?
                .create_consumer(Self::consumer_config(self.instance_id))
                .await?
                .messages()
                .await?;

        tokio::select! {
            _ = self.process_messages(&mut messages_stream) => {
                // When no messages remain, we're done.
            },
            _ = shutdown_token.cancelled() => {
                debug!(task = Self::NAME, "received cancellation");
            }
        }

        self.tracker.close();
        self.tracker.wait().await;
        Ok(())
    }

    pub async fn process_messages(&mut self, messages_stream: &mut Stream) {
        while let Some(msg_result) = messages_stream.next().await {
            match msg_result {
                Ok(msg) => {
                    if let Err(error) = self.process_message(msg).await {
                        warn!(?error, "error processing nats message");
                    }
                }
                Err(error) => {
                    warn!(?error, "error processing nats message stream");
                }
            }
        }
    }

    pub async fn process_message(&mut self, message: Message) -> LayerDbResult<()> {
        message
            .ack()
            .await
            .map_err(|e| LayerDbError::NatsAckRaw(e.to_string()))?;

        let headers = message
            .headers
            .clone()
            .ok_or_else(|| LayerDbError::CacheUpdateNoHeaders)?;
        let event_id = headers
            .get(HEADER_EVENT_ID)
            .ok_or_else(|| LayerDbError::NatsMalformedHeaders)?;
        let cur_chunk = headers
            .get(HEADER_CUR_CHUNK)
            .ok_or_else(|| LayerDbError::NatsMalformedHeaders)?;
        let num_chunks = headers
            .get(HEADER_NUM_CHUNKS)
            .ok_or_else(|| LayerDbError::NatsMalformedHeaders)?;
        let header_size = headers
            .get(HEADER_SIZE)
            .ok_or_else(|| LayerDbError::NatsMissingSizeHeader)?;
        let sending_instance_id = headers
            .get(NATS_HEADER_INSTANCE_ID)
            .ok_or_else(|| LayerDbError::NatsMalformedHeaders)?;

        if self.instance_id.to_string() == sending_instance_id.to_string() {
            return Ok(());
        }

        if num_chunks.as_str() == "1" && cur_chunk.as_str() == "1" {
            let tx = self.tx.clone();
            self.tracker
                .spawn(async move { Self::send_message_as_event(tx, message).await });
        } else {
            // Append bytes from a chunked message into an incomplete buffer, keyed on
            // the event it
            match self.buffers.entry(event_id.to_string()) {
                // Entry found for the event id
                Entry::Occupied(occupied) => {
                    let value = occupied.into_mut();
                    value.put(message.payload.as_ref());
                }
                // Entry not found for the event id
                Entry::Vacant(vacant) => {
                    // Determine the size of the full un-chunked message from header metadata
                    let size: usize = str::parse(header_size.as_str())
                        .map_err(LayerDbError::nats_header_parse)?;

                    let mut buffer = BytesMut::with_capacity(size);
                    buffer.put(message.payload.as_ref());

                    vacant.insert(buffer);
                }
            };

            if cur_chunk == num_chunks {
                // We're at the final message chunk so we can return a reassembled
                // messages

                // Move the payload bytes out of the buffers map
                let payload = self
                    .buffers
                    .remove(event_id.as_str())
                    .ok_or(LayerDbError::MissingInternalBuffer)?;

                let tx = self.tx.clone();
                self.tracker
                    .spawn(async move { Self::send_payload_as_event(tx, payload.into()).await });
            }
        }

        Ok(())
    }

    pub async fn send_message_as_event(
        tx: UnboundedSender<LayeredEvent>,
        message: Message,
    ) -> LayerDbResult<()> {
        let event: LayeredEvent = serialize::from_bytes(&message.payload)?;
        tx.send(event).map_err(Box::new)?;
        Ok(())
    }

    pub async fn send_payload_as_event(
        tx: UnboundedSender<LayeredEvent>,
        payload: Bytes,
    ) -> LayerDbResult<()> {
        let event: LayeredEvent = serialize::from_bytes(&payload)?;
        tx.send(event).map_err(Box::new)?;
        Ok(())
    }

    #[inline]
    fn consumer_config(instance_id: Ulid) -> jetstream::consumer::pull::Config {
        let name = format!("layerdb-event-reader-{instance_id}");
        let description = "layerdb-event-reader for [instance_id]".to_string();

        jetstream::consumer::pull::Config {
            name: Some(name),
            description: Some(description),
            deliver_policy: DeliverPolicy::New,
            max_bytes: MAX_BYTES,
            inactive_threshold: Duration::from_secs(180),
            ..Default::default()
        }
    }
}

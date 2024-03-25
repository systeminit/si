use std::{
    fmt, ops,
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{Future, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use si_data_nats::{
    async_nats::jetstream::{self, message::Acker},
    NatsClient,
};
use strum::EnumDiscriminants;
use ulid::{Ulid, ULID_LEN};

use crate::{
    error::LayerDbResult,
    event::LayeredEventMetadata,
    nats::{self, subject},
    LayerDbError,
};

use self::rebase::{RebaseFinished, RebaseRequest};

pub use si_data_nats::async_nats::jetstream::AckKind;

pub mod rebase;

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ActivityId(Ulid);

impl ActivityId {
    pub fn new() -> ActivityId {
        ActivityId(Ulid::new())
    }

    pub fn array_to_str<'buf>(&self, buf: &'buf mut [u8; ULID_LEN]) -> &'buf mut str {
        self.0.array_to_str(buf)
    }

    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl Default for ActivityId {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for ActivityId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

impl From<ulid::Ulid> for ActivityId {
    fn from(value: ulid::Ulid) -> Self {
        Self(value)
    }
}

impl fmt::Display for ActivityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Activity {
    pub id: ActivityId,
    pub payload: ActivityPayload,
    pub metadata: LayeredEventMetadata,
}

impl Activity {
    pub fn new(payload: ActivityPayload, metadata: LayeredEventMetadata) -> Activity {
        Activity {
            id: ActivityId::new(),
            payload,
            metadata,
        }
    }

    pub fn rebase(request: RebaseRequest, metadata: LayeredEventMetadata) -> Activity {
        Activity::new(ActivityPayload::RebaseRequest(request), metadata)
    }

    pub fn rebase_finished(request: RebaseFinished, metadata: LayeredEventMetadata) -> Activity {
        Activity::new(ActivityPayload::RebaseFinished(request), metadata)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumDiscriminants, PartialEq, Eq)]
pub enum ActivityPayload {
    RebaseRequest(RebaseRequest),
    RebaseFinished(RebaseFinished),
}

impl ActivityPayload {
    pub fn to_subject(&self) -> String {
        let discriminate: ActivityPayloadDiscriminants = self.into();
        discriminate.to_subject()
    }
}

impl ActivityPayloadDiscriminants {
    pub fn to_subject(&self) -> String {
        match self {
            ActivityPayloadDiscriminants::RebaseRequest => "rebase.request".to_string(),
            ActivityPayloadDiscriminants::RebaseFinished => "rebase.finished".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct AckActivity {
    inner: Activity,
    acker: Arc<Acker>,
}

impl AckActivity {
    pub fn into_parts(self) -> (Activity, Arc<Acker>) {
        (self.inner, self.acker)
    }

    pub fn as_activity(&self) -> &Activity {
        &self.inner
    }

    pub async fn ack(&self) -> LayerDbResult<()> {
        self.acker.ack().await.map_err(LayerDbError::NatsAck)
    }

    pub async fn ack_with(&self, kind: AckKind) -> LayerDbResult<()> {
        self.acker
            .ack_with(kind)
            .await
            .map_err(LayerDbError::NatsAck)
    }

    pub async fn double_ack(&self) -> LayerDbResult<()> {
        self.acker.double_ack().await.map_err(LayerDbError::NatsAck)
    }
}

impl fmt::Debug for AckActivity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AckActivity")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl PartialEq for AckActivity {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl ops::Deref for AckActivity {
    type Target = Activity;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct ActivityPublisher {
    prefix: Option<Arc<str>>,
    context: jetstream::context::Context,
}

impl ActivityPublisher {
    pub(crate) fn new(nats_client: &NatsClient) -> ActivityPublisher {
        let prefix = nats_client.metadata().subject_prefix().map(|s| s.into());
        let context = jetstream::new(nats_client.as_inner().clone());
        ActivityPublisher { context, prefix }
    }

    pub(crate) async fn publish(&self, activity: &Activity) -> LayerDbResult<()> {
        let nats_subject = subject::for_activity(self.prefix(), activity);
        let nats_payload = postcard::to_stdvec(&activity)?;
        // Publish message and await confirmation from server that it has been received
        self.context
            .publish(nats_subject, nats_payload.into())
            .await?
            .await?;
        Ok(())
    }

    fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
}

pub struct ActivityStream {
    inner: jetstream::consumer::pull::Stream,
}

impl ActivityStream {
    pub(crate) async fn create(
        instance_id: Ulid,
        nats_client: &NatsClient,
        filters: Option<impl IntoIterator<Item = ActivityPayloadDiscriminants>>,
    ) -> LayerDbResult<Self> {
        let context = jetstream::new(nats_client.as_inner().clone());

        let inner =
            nats::layerdb_activities_stream(&context, nats_client.metadata().subject_prefix())
                .await?
                .create_consumer(Self::consumer_config(
                    nats_client.metadata().subject_prefix(),
                    instance_id,
                    filters,
                ))
                .await?
                .messages()
                .await?;

        Ok(Self { inner })
    }

    #[inline]
    fn consumer_config(
        prefix: Option<&str>,
        instance_id: Ulid,
        filters: Option<impl IntoIterator<Item = ActivityPayloadDiscriminants>>,
    ) -> jetstream::consumer::pull::Config {
        let name = format!("activity-stream-{instance_id}");
        let description = format!("activity stream for [{name}]");

        let mut config = jetstream::consumer::pull::Config {
            name: Some(name),
            description: Some(description),
            deliver_policy: jetstream::consumer::DeliverPolicy::New,
            ..Default::default()
        };

        if let Some(payload_types) = filters {
            config.filter_subjects = payload_types
                .into_iter()
                .map(|t| nats::subject::for_activity_discriminate(prefix, t))
                .map(|s| s.to_string())
                .collect();
        }

        config
    }
}

impl Stream for ActivityStream {
    type Item = LayerDbResult<AckActivity>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner.next()).poll(cx) {
            // Process the message
            Poll::Ready(Some(Ok(msg))) => {
                let (msg, acker) = msg.split();

                match postcard::from_bytes::<Activity>(&msg.payload) {
                    // Successfully deserialized into an activity
                    Ok(inner) => Poll::Ready(Some(Ok(AckActivity {
                        inner,
                        acker: Arc::new(acker),
                    }))),
                    // Error deserializing message
                    Err(err) => Poll::Ready(Some(Err(err.into()))),
                }
            }
            // Upstream errors are propagated downstream
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
            // If the upstream closes, then we do too
            Poll::Ready(None) => Poll::Ready(None),
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Clone)]
pub struct AckRebaseRequest {
    pub id: ActivityId,
    pub payload: RebaseRequest,
    pub metadata: LayeredEventMetadata,
    acker: Arc<Acker>,
}

impl AckRebaseRequest {
    pub async fn ack(&self) -> LayerDbResult<()> {
        self.acker.ack().await.map_err(LayerDbError::NatsAck)
    }

    pub async fn ack_with(&self, kind: AckKind) -> LayerDbResult<()> {
        self.acker
            .ack_with(kind)
            .await
            .map_err(LayerDbError::NatsAck)
    }

    pub async fn double_ack(&self) -> LayerDbResult<()> {
        self.acker.double_ack().await.map_err(LayerDbError::NatsAck)
    }
}

pub struct RebaserRequestsWorkQueueStream {
    inner: jetstream::consumer::pull::Stream,
}

impl RebaserRequestsWorkQueueStream {
    const CONSUMER_NAME: &'static str = "rebaser-requests";

    pub(crate) async fn create(nats_client: &NatsClient) -> LayerDbResult<Self> {
        let context = jetstream::new(nats_client.as_inner().clone());

        // Ensure the sourced stream is created
        let _activities =
            nats::layerdb_activities_stream(&context, nats_client.metadata().subject_prefix())
                .await?;

        let inner = nats::rebaser_requests_work_queue_stream(
            &context,
            nats_client.metadata().subject_prefix(),
        )
        .await?
        .create_consumer(Self::consumer_config())
        .await?
        .messages()
        .await?;

        Ok(Self { inner })
    }

    #[inline]
    fn consumer_config() -> jetstream::consumer::pull::Config {
        jetstream::consumer::pull::Config {
            durable_name: Some(Self::CONSUMER_NAME.to_string()),
            description: Some("rebaser requests consumer".to_string()),
            ..Default::default()
        }
    }
}

impl Stream for RebaserRequestsWorkQueueStream {
    type Item = LayerDbResult<AckRebaseRequest>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner.next()).poll(cx) {
            // Process the message
            Poll::Ready(Some(Ok(msg))) => {
                let (msg, acker) = msg.split();

                match postcard::from_bytes::<Activity>(&msg.payload) {
                    // Successfully deserialized into an activity
                    Ok(activity) => match activity.payload {
                        // Correct variant, convert to work-specific type
                        ActivityPayload::RebaseRequest(req) => {
                            Poll::Ready(Some(Ok(AckRebaseRequest {
                                id: activity.id,
                                payload: req,
                                metadata: activity.metadata,
                                acker: Arc::new(acker),
                            })))
                        }
                        // Unexpected variant, message is invalid
                        _ => Poll::Ready(Some(Err(LayerDbError::UnexpectedActivityVariant(
                            ActivityPayloadDiscriminants::RebaseRequest.to_subject(),
                            ActivityPayloadDiscriminants::from(activity.payload).to_subject(),
                        )))),
                    },
                    // Error deserializing message
                    Err(err) => Poll::Ready(Some(Err(err.into()))),
                }
            }
            // Upstream errors are propagated downstream
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
            // If the upstream closes, then we do too
            Poll::Ready(None) => Poll::Ready(None),
            // Not ready, so...not ready!
            Poll::Pending => Poll::Pending,
        }
    }
}

use std::{
    collections::HashMap,
    fmt::{self, Debug},
    ops,
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
use tokio::sync::{broadcast, RwLock};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::{Ulid, ULID_LEN};

use crate::{
    error::LayerDbResult,
    event::LayeredEventMetadata,
    nats::{self, subject},
    LayerDbError,
};

use self::{
    rebase::{RebaseFinished, RebaseRequest},
    test::{IntegrationTest, IntegrationTestAlt},
};

use telemetry::prelude::*;

pub use si_data_nats::async_nats::jetstream::AckKind;

pub mod rebase;
pub mod test;

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
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Activity {
    pub id: ActivityId,
    pub payload: ActivityPayload,
    pub metadata: LayeredEventMetadata,
    pub parent_activity_id: Option<ActivityId>,
}

impl Activity {
    pub fn new(
        payload: ActivityPayload,
        metadata: LayeredEventMetadata,
        parent_activity_id: Option<ActivityId>,
    ) -> Activity {
        Activity {
            id: ActivityId::new(),
            payload,
            metadata,
            parent_activity_id,
        }
    }

    pub fn rebase(request: RebaseRequest, metadata: LayeredEventMetadata) -> Activity {
        Activity::new(ActivityPayload::RebaseRequest(request), metadata, None)
    }

    pub fn rebase_finished(
        request: RebaseFinished,
        metadata: LayeredEventMetadata,
        from_rebase_activity_id: ActivityId,
    ) -> Activity {
        Activity::new(
            ActivityPayload::RebaseFinished(request),
            metadata,
            Some(from_rebase_activity_id),
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumDiscriminants, PartialEq, Eq)]
pub enum ActivityPayload {
    RebaseRequest(RebaseRequest),
    RebaseFinished(RebaseFinished),
    IntegrationTest(IntegrationTest),
    IntegrationTestAlt(IntegrationTestAlt),
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
            ActivityPayloadDiscriminants::IntegrationTest => "integration_test.test".to_string(),
            ActivityPayloadDiscriminants::IntegrationTestAlt => "integration_test.alt".to_string(),
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

#[derive(Debug, Clone)]
pub struct ActivityMultiplexer {
    instance_id: Ulid,
    nats_client: NatsClient,
    tracker: TaskTracker,
    shutdown_token: CancellationToken,
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Activity>>>>,
}

impl ActivityMultiplexer {
    pub fn new(
        instance_id: Ulid,
        nats_client: NatsClient,

        shutdown_token: CancellationToken,
    ) -> Self {
        let tracker = TaskTracker::new();
        Self {
            tracker,
            instance_id,
            nats_client,
            shutdown_token,
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn subscribe(
        &self,
        filters: Option<impl IntoIterator<Item = ActivityPayloadDiscriminants>>,
    ) -> LayerDbResult<broadcast::Receiver<Activity>> {
        let (multiplex_key, has_filter_array) = if let Some(filters) = filters {
            let filter_array: Vec<ActivityPayloadDiscriminants> = filters.into_iter().collect();
            (
                filter_array
                    .iter()
                    .map(|d| d.to_subject())
                    .collect::<Vec<String>>()
                    .join("."),
                Some(filter_array),
            )
        } else {
            ("everything".to_string(), None)
        };
        {
            let reader = self.channels.read().await;
            if let Some(sender) = reader.get(&multiplex_key) {
                return Ok(sender.subscribe());
            }
        }
        let activity_stream =
            ActivityStream::create(self.instance_id, &self.nats_client, has_filter_array).await?;
        let (tx, rx) = broadcast::channel(1000); // the 1000 here is the depth the channel will
                                                 // keep if a reader is slow
        let mut amx_task = ActivityMultiplexerTask::new(activity_stream, tx.clone());
        let amx_shutdown_token = self.shutdown_token.clone();
        self.tracker
            .spawn(async move { amx_task.run(amx_shutdown_token).await });
        {
            let mut writer = self.channels.write().await;
            writer.insert(multiplex_key, tx);
        }
        Ok(rx)
    }
}

pub struct ActivityMultiplexerTask {
    activity_stream: ActivityStream,
    tx: broadcast::Sender<Activity>,
}

impl ActivityMultiplexerTask {
    pub fn new(activity_stream: ActivityStream, tx: broadcast::Sender<Activity>) -> Self {
        Self {
            activity_stream,
            tx,
        }
    }

    pub async fn run(&mut self, token: CancellationToken) -> LayerDbResult<()> {
        tokio::select! {
            () = self.process() => {
                debug!("activity multiplexer task has ended; likely a bug");
            },
            () = token.cancelled() => {
                debug!("activity multiplexer has been cancelled; shutting down");
            },
        }
        Ok(())
    }

    pub async fn process(&mut self) {
        while let Some(ack_activity_result) = self.activity_stream.next().await {
            match ack_activity_result {
                Ok(ack_activity) => match ack_activity.ack().await {
                    Ok(_) => {
                        if let Err(e) = self.tx.send(ack_activity.inner) {
                            trace!(
                                ?e,
                                "activity multiplexer skipping message; no receivers listening. this can be totally normal!"
                            );
                        }
                    }
                    Err(e) => warn!(?e, "Failed to ack an activity stream message; bug!"),
                },
                Err(e) => {
                    warn!(?e, "Activity stream message had an error; bug!");
                }
            }
        }
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
                    Err(err) => {
                        error!(?msg, "failure to deserialize message in activity stream");
                        Poll::Ready(Some(Err(err.into())))
                    }
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

impl Debug for AckRebaseRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AckRebaseRequest {{ id: {:?}, payload: {:?}, metadata: {:?} }}",
            self.id, self.payload, self.metadata
        )
    }
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
                            warn!(?req, "received rebase request over pull channel");
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
                    Err(err) => {
                        warn!(?msg, "failed to deserialize message");
                        Poll::Ready(Some(Err(err.into())))
                    }
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

use std::{fmt, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use si_data_nats::{async_nats::jetstream, NatsClient};
use strum::EnumDiscriminants;
use ulid::{Ulid, ULID_LEN};

use crate::{
    error::LayerDbResult,
    event::LayeredEventMetadata,
    nats::{self, subject},
};

use self::rebase::{RebaseFinished, RebaseRequest};

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

#[derive(Debug, Clone)]
pub struct ActivityPublisher {
    prefix: Option<Arc<str>>,
    context: jetstream::context::Context,
}

impl ActivityPublisher {
    pub fn new(nats_client: &NatsClient) -> ActivityPublisher {
        let prefix = nats_client.metadata().subject_prefix().map(|s| s.into());
        let context = jetstream::new(nats_client.as_inner().clone());
        ActivityPublisher { context, prefix }
    }

    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    pub fn publish(&self, activity: &Activity) -> LayerDbResult<()> {
        let nats_subject = subject::for_activity(self.prefix(), activity);
        let nats = self.context.clone();
        let nats_payload = postcard::to_stdvec(&activity)?;
        let _nats_join =
            tokio::spawn(async move { nats.publish(nats_subject, nats_payload.into()).await });
        Ok(())
    }
}

#[allow(dead_code)]
pub struct ActivitySubscriber {
    instance_id: Ulid,
    messages: jetstream::consumer::pull::Stream,
}

impl ActivitySubscriber {
    pub async fn new(
        instance_id: Ulid,
        nats_client: &NatsClient,
        to_receive: Option<Vec<ActivityPayloadDiscriminants>>,
    ) -> LayerDbResult<ActivitySubscriber> {
        let context = jetstream::new(nats_client.as_inner().clone());

        let activities =
            nats::layerdb_activities_stream(&context, nats_client.metadata().subject_prefix())
                .await?
                .create_consumer(Self::consumer_config(
                    nats_client.metadata().subject_prefix(),
                    instance_id,
                    to_receive,
                ))
                .await?
                .messages()
                .await?;

        Ok(ActivitySubscriber {
            instance_id,
            messages: activities,
        })
    }

    pub fn messages(&mut self) -> &mut jetstream::consumer::pull::Stream {
        &mut self.messages
    }

    #[inline]
    fn consumer_config(
        prefix: Option<&str>,
        instance_id: Ulid,
        to_receive: Option<Vec<ActivityPayloadDiscriminants>>,
    ) -> jetstream::consumer::pull::Config {
        let name = format!("activity-stream-{instance_id}");
        let description = format!("activity stream for [{name}]");

        match to_receive {
            Some(payload_types) => jetstream::consumer::pull::Config {
                name: Some(name),
                description: Some(description),
                deliver_policy: jetstream::consumer::DeliverPolicy::New,
                filter_subjects: payload_types
                    .iter()
                    .map(|t| nats::subject::for_activity_discriminate(prefix, t))
                    .map(|s| s.to_string())
                    .collect(),
                ..Default::default()
            },
            None => jetstream::consumer::pull::Config {
                name: Some(name),
                description: Some(description),
                deliver_policy: jetstream::consumer::DeliverPolicy::New,
                ..Default::default()
            },
        }
    }
}

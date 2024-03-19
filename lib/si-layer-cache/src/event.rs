use std::sync::Arc;

use chrono::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use si_events::{Actor, Tenancy, WebEvent};
use strum::AsRefStr;
use ulid::Ulid;

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

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LayeredEventId(Ulid);

impl Default for LayeredEventId {
    fn default() -> Self {
        Self::new()
    }
}

impl LayeredEventId {
    pub fn new() -> Self {
        LayeredEventId(Ulid::new())
    }

    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl std::str::FromStr for LayeredEventId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

#[derive(AsRefStr, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LayeredEventKind {
    CasInsertion,
    Raw,
    SnapshotWrite,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct LayeredEvent {
    pub event_id: LayeredEventId,
    pub event_kind: LayeredEventKind,
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
            metadata: LayeredEventMetadata::new(tenancy, actor),
            payload: LayeredEventPayload::new(db_name, key, value, sort_key),
            web_events,
        }
    }
}

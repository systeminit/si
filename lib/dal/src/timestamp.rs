use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum TimestampError {}

pub type TimestampResult<T> = Result<T, TimestampError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Timestamp {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Timestamp {
    pub fn now() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }

    pub fn assemble(created_at: DateTime<Utc>, updated_at: DateTime<Utc>) -> Self {
        Self {
            created_at,
            updated_at,
        }
    }

    pub fn set_updated(&mut self) {
        self.updated_at = Utc::now();
    }
}

impl From<si_events::Timestamp> for Timestamp {
    fn from(value: si_events::Timestamp) -> Self {
        Self {
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Timestamp> for si_events::Timestamp {
    fn from(value: Timestamp) -> Self {
        Self {
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

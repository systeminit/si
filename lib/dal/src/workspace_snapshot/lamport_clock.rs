//! Lamport Clocks

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::workspace_snapshot::ChangeSetError;

#[derive(Debug, Error)]
pub enum LamportClockError {
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
}

pub type LamportClockResult<T> = Result<T, LamportClockError>;

#[derive(Clone, Copy, Deserialize, Serialize, Ord, Eq, PartialEq, PartialOrd)]
pub struct LamportClock {
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub counter: DateTime<Utc>,
}

impl Default for LamportClock {
    fn default() -> Self {
        Self::new()
    }
}

impl LamportClock {
    pub fn new() -> Self {
        let counter = Utc::now();
        Self { counter }
    }

    pub fn new_with_value(new_value: DateTime<Utc>) -> Self {
        LamportClock { counter: new_value }
    }

    pub fn inc(&mut self) {
        self.counter = Utc::now();
    }

    pub fn inc_to(&mut self, new_value: DateTime<Utc>) {
        self.counter = new_value;
    }

    pub fn inc_to_max_of(&mut self, new_value: DateTime<Utc>) {
        if new_value > self.counter {
            self.counter = new_value;
        }
    }

    pub fn merge(&mut self, other: &LamportClock) {
        if self.counter < other.counter {
            self.counter = other.counter;
        }
    }
}

impl std::fmt::Debug for LamportClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LamportClock({})", &self.counter.to_string())
    }
}

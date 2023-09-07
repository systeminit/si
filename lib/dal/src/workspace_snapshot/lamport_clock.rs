//! Lamport Clocks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

use crate::workspace_snapshot::{ChangeSetPointer, ChangeSetPointerError};

#[derive(Debug, Error)]
pub enum LamportClockError {
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
}

pub type LamportClockResult<T> = Result<T, LamportClockError>;

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct LamportClock {
    #[serde(with = "chrono::serde::ts_nanoseconds")]
    pub counter: DateTime<Utc>,
}

impl LamportClock {
    pub fn new() -> LamportClockResult<LamportClock> {
        let counter = Utc::now();
        Ok(LamportClock { counter })
    }

    pub fn new_with_value(new_value: DateTime<Utc>) -> Self {
        LamportClock { counter: new_value }
    }

    pub fn inc(&mut self) -> LamportClockResult<()> {
        self.counter = Utc::now();

        Ok(())
    }

    pub fn inc_to(&mut self, new_value: DateTime<Utc>) {
        self.counter = new_value;
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

impl Eq for LamportClock {}

impl PartialEq for LamportClock {
    fn eq(&self, other: &Self) -> bool {
        self.counter == other.counter
    }
}

impl PartialOrd for LamportClock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.counter.partial_cmp(&other.counter)
    }
}

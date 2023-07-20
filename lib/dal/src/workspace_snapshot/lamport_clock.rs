//! Lamport Clocks

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

use crate::workspace_snapshot::{ChangeSet, ChangeSetError};

#[derive(Debug, Error)]
pub enum LamportClockError {
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
}

pub type LamportClockResult<T> = Result<T, LamportClockError>;

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct LamportClock {
    pub counter: Ulid,
}

impl LamportClock {
    pub fn new(change_set: &ChangeSet) -> LamportClockResult<LamportClock> {
        let counter = change_set.generate_ulid()?;
        Ok(LamportClock { counter })
    }

    pub fn inc(&mut self, change_set: &ChangeSet) -> LamportClockResult<()> {
        let next = change_set.generate_ulid()?;
        self.counter = next;

        Ok(())
    }

    pub fn merge(&mut self, other: &LamportClock) {
        if self.counter < other.counter {
            self.counter = other.counter;
        }
    }
}

impl std::fmt::Debug for LamportClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LamportClock")
            .field("counter", &self.counter.to_string())
            .finish()
    }
}

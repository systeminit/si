//! Lamport Clocks

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::workspace_snapshot::ChangeSet;

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct LamportClock {
    pub counter: Ulid,
}

impl LamportClock {
    pub fn new(change_set: &ChangeSet) -> LamportClock {
        let counter = change_set.generator.lock().unwrap().generate().unwrap();
        LamportClock { counter }
    }

    pub fn inc(&mut self, change_set: &ChangeSet) {
        let next = change_set.generator.lock().unwrap().generate().unwrap();
        self.counter = next;
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
            .field("counter", &self.counter)
            .finish()
    }
}

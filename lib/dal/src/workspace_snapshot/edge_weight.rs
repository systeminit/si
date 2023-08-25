//! Edges

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::workspace_snapshot::{
    change_set::ChangeSet,
    vector_clock::{VectorClock, VectorClockError},
};

#[derive(Debug, Error)]
pub enum EdgeWeightError {
    #[error("Vector Clock error: {0}")]
    VectorClock(#[from] VectorClockError),
}

pub type EdgeWeightResult<T> = Result<T, EdgeWeightError>;

#[derive(Default, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeWeightKind {
    /// Used to record the order that the elements of a container should be presented in.
    Ordering,
    /// Workspaces "use" functions, modules, schemas. Schemas "use" schema variants.
    /// Schema variants "use" props. Props "use" functions, and other props. Modules
    /// "use" functions, schemas, and eventually(?) components.
    #[default]
    Uses,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EdgeWeight {
    kind: EdgeWeightKind,
    vector_clock_first_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl EdgeWeight {
    pub fn increment_vector_clocks(&mut self, change_set: &ChangeSet) -> EdgeWeightResult<()> {
        self.vector_clock_write.inc(change_set)?;

        Ok(())
    }

    pub fn kind(&self) -> EdgeWeightKind {
        self.kind
    }

    pub fn mark_seen_at(&mut self, change_set: &ChangeSet, seen_at: DateTime<Utc>) {
        if self.vector_clock_first_seen.entry_for(change_set).is_none() {
            self.vector_clock_first_seen.inc_to(change_set, seen_at);
        }
    }

    pub fn new(change_set: &ChangeSet, kind: EdgeWeightKind) -> EdgeWeightResult<Self> {
        Ok(Self {
            kind,
            vector_clock_first_seen: VectorClock::new(change_set)?,
            vector_clock_write: VectorClock::new(change_set)?,
        })
    }

    pub fn new_with_incremented_vector_clocks(
        &self,
        change_set: &ChangeSet,
    ) -> EdgeWeightResult<Self> {
        let mut new_weight = self.clone();
        new_weight.increment_vector_clocks(change_set)?;

        Ok(new_weight)
    }

    pub fn vector_clock_first_seen(&self) -> &VectorClock {
        &self.vector_clock_first_seen
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        &self.vector_clock_write
    }
}

//! Edges

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::change_set_pointer::ChangeSetPointer;
use crate::workspace_snapshot::vector_clock::{VectorClock, VectorClockError};

#[derive(Debug, Error)]
pub enum EdgeWeightError {
    #[error("Vector Clock error: {0}")]
    VectorClock(#[from] VectorClockError),
}

pub type EdgeWeightResult<T> = Result<T, EdgeWeightError>;

#[remain::sorted]
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum EdgeWeightKind {
    /// An argument to a function defined by an [`AttributePrototype`][crate::AttributePrototype],
    /// including the name of the argument to the function.
    Argument(String),
    /// An [`AttributeValue`] "contained" by another [`AttributeValue`], such as an entry in an
    /// array/map, or a field of an object. The optional [`String`] represents the key of the entry
    /// in a map.
    Contain(Option<String>),
    /// Used when the target/destination of an edge is an [`InternalProvider`], or an
    /// [`ExternalProvider`].
    DataProvider,
    /// Used to record the order that the elements of a container should be presented in.
    Ordering,
    Prop,
    Prototype,
    Proxy,
    /// Workspaces "use" functions, modules, schemas. Schemas "use" schema variants.
    /// Schema variants "use" props. Props "use" functions, and other props. Modules
    /// "use" functions, schemas, and eventually(?) components.
    #[default]
    Use,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EdgeWeight {
    kind: EdgeWeightKind,
    vector_clock_first_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl EdgeWeight {
    pub fn increment_vector_clocks(
        &mut self,
        change_set: &ChangeSetPointer,
    ) -> EdgeWeightResult<()> {
        self.vector_clock_write.inc(change_set)?;

        Ok(())
    }

    pub fn kind(&self) -> &EdgeWeightKind {
        &self.kind
    }

    pub fn mark_seen_at(&mut self, change_set: &ChangeSetPointer, seen_at: DateTime<Utc>) {
        if self.vector_clock_first_seen.entry_for(change_set).is_none() {
            self.vector_clock_first_seen.inc_to(change_set, seen_at);
        }
    }

    pub fn new(change_set: &ChangeSetPointer, kind: EdgeWeightKind) -> EdgeWeightResult<Self> {
        Ok(Self {
            kind,
            vector_clock_first_seen: VectorClock::new(change_set)?,
            vector_clock_write: VectorClock::new(change_set)?,
        })
    }

    pub fn new_with_incremented_vector_clocks(
        &self,
        change_set: &ChangeSetPointer,
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

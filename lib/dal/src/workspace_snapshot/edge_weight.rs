//! Edges

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::change_set_pointer::ChangeSetPointer;
use crate::workspace_snapshot::vector_clock::{VectorClock, VectorClockError, VectorClockId};
use crate::ActionKind;

use strum::EnumDiscriminants;

#[derive(Debug, Error)]
pub enum EdgeWeightError {
    #[error("Vector Clock error: {0}")]
    VectorClock(#[from] VectorClockError),
}

pub type EdgeWeightResult<T> = Result<T, EdgeWeightError>;

#[remain::sorted]
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize))]
pub enum EdgeWeightKind {
    /// A function used by a [`SchemaVariant`] to perform an action that affects its resource
    ActionPrototype(ActionKind),
    /// An [`AttributeValue`] "contained" by another [`AttributeValue`], such as an entry in an
    /// array/map, or a field of an object. The optional [`String`] represents the key of the entry
    /// in a map.
    Contain(Option<String>),
    /// Used to record the order that the elements of a container should be presented in.
    Ordering,
    /// Connects the node at the Ordering edge directly to the things it orders.
    Ordinal,
    /// Used to link an attribute value to the prop that it is for.
    Prop,
    /// An edge from a [`provider`](crate::provider) to an
    /// [`AttributePrototype`](crate::AttributePrototype). The optional [`String`] is used for
    /// maps, arrays and relevant container types to indicate which element the prototype is for.
    Prototype(Option<String>),
    /// An edge from an [`AttributePrototype`][crate::AttributePrototype] to an
    /// [`AttributePrototypeArgument`][crate::AttributePrototypeArgument].
    PrototypeArgument,
    /// An edge from an
    /// [`AttributePrototypeArgument`][crate::AttributePrototypeArgument] to the
    /// source for the value for this argument
    PrototypeArgumentValue,
    /// Used when the target/destination of an edge is an [`InternalProvider`], or an
    /// [`ExternalProvider`].
    Provider,
    Proxy,
    /// Indicates the "root" [`AttributeValue`](crate::AttributeValue) for a [`Component`](crate::Component).
    ///
    /// TODO(nick): in the future, this should be used for the "root" [`Prop`](crate::Prop) for a
    /// [`SchemaVariant`](crate::SchemaVariant) as well.
    Root,
    /// Edge from component to input or output Socket's attribute value
    Socket,
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
        self.vector_clock_write.inc(change_set.vector_clock_id())?;

        Ok(())
    }

    pub fn kind(&self) -> &EdgeWeightKind {
        &self.kind
    }

    pub fn mark_seen_at(&mut self, vector_clock_id: VectorClockId, seen_at: DateTime<Utc>) {
        if self
            .vector_clock_first_seen
            .entry_for(vector_clock_id)
            .is_none()
        {
            self.vector_clock_first_seen
                .inc_to(vector_clock_id, seen_at);
        }
    }

    pub fn new(change_set: &ChangeSetPointer, kind: EdgeWeightKind) -> EdgeWeightResult<Self> {
        Ok(Self {
            kind,
            vector_clock_first_seen: VectorClock::new(change_set.vector_clock_id())?,
            vector_clock_write: VectorClock::new(change_set.vector_clock_id())?,
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

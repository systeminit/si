//! Edges

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use thiserror::Error;

use crate::change_set_pointer::ChangeSetPointer;
use crate::workspace_snapshot::vector_clock::{VectorClock, VectorClockError, VectorClockId};
use crate::ActionKind;

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
    Action,
    /// A function used by a [`SchemaVariant`] to perform an action that affects its resource
    ActionPrototype(ActionKind),
    /// A function defined for a secret defining [`SchemaVariant`] to be executed before funcs on
    /// components that have a secret of that kind
    AuthenticationPrototype,
    /// An [`AttributeValue`] "contained" by another [`AttributeValue`], such as an entry in an
    /// array/map, or a field of an object. The optional [`String`] represents the key of the entry
    /// in a map.
    Contain(Option<String>),
    /// Used to denote when something is a default. For example a default schema variant for a schema
    Default,
    /// Used to indicate parentage within frames. It does not dictate data flow. That is provided via
    /// [`ComponentType`](crate::ComponentType).
    ///
    /// This replaces "Symbolic" edges and "Frame" sockets from the old engine.
    FrameContains,
    /// Used to record the order that the elements of a container should be presented in.
    Ordering,
    /// Connects the node at the Ordering edge directly to the things it orders.
    Ordinal,
    /// Used to link an attribute value to the prop that it is for.
    Prop,
    /// An edge from a [`socket`](crate::socket) or an [`AttributeValue`](`crate::AttributeValue`)
    /// to an [`AttributePrototype`](crate::AttributePrototype). The optional [`String`] is used for
    /// maps, arrays and relevant container types to indicate which element the prototype is for.
    Prototype(Option<String>),
    /// An edge from an [`AttributePrototype`][crate::AttributePrototype] to an
    /// [`AttributePrototypeArgument`][crate::AttributePrototypeArgument].
    PrototypeArgument,
    /// An edge from an
    /// [`AttributePrototypeArgument`][crate::AttributePrototypeArgument] to the
    /// source for the value for this argument
    PrototypeArgumentValue,
    Proxy,
    /// Indicates the "root" [`AttributeValue`](crate::AttributeValue) for a [`Component`](crate::Component).
    ///
    /// TODO(nick): in the future, this should be used for the "root" [`Prop`](crate::Prop) for a
    /// [`SchemaVariant`](crate::SchemaVariant) as well.
    Root,
    /// Used when the target/destination of an edge is an [`InputSocket`](crate::InputSocket), or an
    /// [`OutputSocket`](crate::OutputSocket).
    Socket,
    /// Edge from component to input or output Socket's attribute value
    SocketValue,
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

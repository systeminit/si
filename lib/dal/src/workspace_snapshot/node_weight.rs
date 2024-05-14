use std::num::TryFromIntError;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    EncryptedSecretKey,
    {ulid::Ulid, ContentHash},
};
use strum::EnumDiscriminants;
use thiserror::Error;

use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::{
    action::prototype::ActionKind,
    change_set::{ChangeSet, ChangeSetError},
    workspace_snapshot::{
        content_address::ContentAddress,
        vector_clock::{VectorClock, VectorClockError},
    },
    ChangeSetId, PropKind,
};
use crate::{func::execution::FuncExecutionPk, EdgeWeightKindDiscriminants};

use crate::func::FuncKind;
use crate::workspace_snapshot::node_weight::secret_node_weight::SecretNodeWeight;
pub use action_node_weight::ActionNodeWeight;
pub use action_prototype_node_weight::ActionPrototypeNodeWeight;
pub use attribute_prototype_argument_node_weight::ArgumentTargets;
pub use attribute_prototype_argument_node_weight::AttributePrototypeArgumentNodeWeight;
pub use attribute_value_node_weight::AttributeValueNodeWeight;
pub use category_node_weight::CategoryNodeWeight;
pub use component_node_weight::ComponentNodeWeight;
pub use content_node_weight::ContentNodeWeight;
pub use func_argument_node_weight::FuncArgumentNodeWeight;
pub use func_node_weight::FuncNodeWeight;
pub use ordering_node_weight::OrderingNodeWeight;
pub use prop_node_weight::PropNodeWeight;

use super::content_address::ContentAddressDiscriminants;

pub mod action_node_weight;
pub mod action_prototype_node_weight;
pub mod attribute_prototype_argument_node_weight;
pub mod attribute_value_node_weight;
pub mod category_node_weight;
pub mod component_node_weight;
pub mod content_node_weight;
pub mod func_argument_node_weight;
pub mod func_node_weight;
pub mod ordering_node_weight;
pub mod prop_node_weight;
pub mod secret_node_weight;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum NodeWeightError {
    #[error("Cannot set content hash directly on node weight kind")]
    CannotSetContentHashOnKind,
    #[error("Cannot set content order directly on node weight kind")]
    CannotSetOrderOnKind,
    #[error("Cannot update root node's content hash")]
    CannotUpdateRootNodeContentHash,
    #[error("ChangeSet error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("Incompatible node weights")]
    IncompatibleNodeWeightVariants,
    #[error("Invalid ContentAddress variant ({0}) for NodeWeight variant ({1})")]
    InvalidContentAddressForWeightKind(String, String),
    #[error("Missing Key for Child Entry {0}")]
    MissingKeytForChildEntry(Ulid),
    #[error("try from int error: {0}")]
    TryFromIntError(#[from] TryFromIntError),
    #[error("Unexpected content address variant: {1} expected {0}")]
    UnexpectedContentAddressVariant(ContentAddressDiscriminants, ContentAddressDiscriminants),
    #[error("Unexpected node weight variant. Got {1} but expected {0}")]
    UnexpectedNodeWeightVariant(NodeWeightDiscriminants, NodeWeightDiscriminants),
    #[error("Vector Clock error: {0}")]
    VectorClock(#[from] VectorClockError),
}

pub type NodeWeightResult<T> = Result<T, NodeWeightError>;

#[derive(Debug, Serialize, Deserialize, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize))]
pub enum NodeWeight {
    Action(ActionNodeWeight),
    ActionPrototype(ActionPrototypeNodeWeight),
    AttributePrototypeArgument(AttributePrototypeArgumentNodeWeight),
    AttributeValue(AttributeValueNodeWeight),
    Category(CategoryNodeWeight),
    Component(ComponentNodeWeight),
    Content(ContentNodeWeight),
    Func(FuncNodeWeight),
    FuncArgument(FuncArgumentNodeWeight),
    Ordering(OrderingNodeWeight),
    Prop(PropNodeWeight),
    Secret(SecretNodeWeight),
}

impl NodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            NodeWeight::Action(weight) => weight.content_hash(),
            NodeWeight::ActionPrototype(weight) => weight.content_hash(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.content_hash(),
            NodeWeight::AttributeValue(weight) => weight.content_hash(),
            NodeWeight::Category(weight) => weight.content_hash(),
            NodeWeight::Component(weight) => weight.content_hash(),
            NodeWeight::Content(weight) => weight.content_hash(),
            NodeWeight::Func(weight) => weight.content_hash(),
            NodeWeight::FuncArgument(weight) => weight.content_hash(),
            NodeWeight::Ordering(weight) => weight.content_hash(),
            NodeWeight::Prop(weight) => weight.content_hash(),
            NodeWeight::Secret(weight) => weight.content_hash(),
        }
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        match self {
            NodeWeight::Action(weight) => weight.content_store_hashes(),
            NodeWeight::ActionPrototype(weight) => weight.content_store_hashes(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.content_store_hashes(),
            NodeWeight::AttributeValue(weight) => weight.content_store_hashes(),
            NodeWeight::Category(weight) => weight.content_store_hashes(),
            NodeWeight::Component(weight) => weight.content_store_hashes(),
            NodeWeight::Content(weight) => weight.content_store_hashes(),
            NodeWeight::Func(weight) => weight.content_store_hashes(),
            NodeWeight::FuncArgument(weight) => weight.content_store_hashes(),
            NodeWeight::Ordering(weight) => weight.content_store_hashes(),
            NodeWeight::Prop(weight) => weight.content_store_hashes(),
            NodeWeight::Secret(weight) => weight.content_store_hashes(),
        }
    }

    pub fn content_address_discriminants(&self) -> Option<ContentAddressDiscriminants> {
        match self {
            NodeWeight::Content(weight) => Some(weight.content_address().into()),
            NodeWeight::Action(_)
            | NodeWeight::ActionPrototype(_)
            | NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::Component(_)
            | NodeWeight::Func(_)
            | NodeWeight::FuncArgument(_)
            | NodeWeight::Ordering(_)
            | NodeWeight::Prop(_)
            | NodeWeight::Secret(_) => None,
        }
    }

    pub fn id(&self) -> Ulid {
        match self {
            NodeWeight::Action(weight) => weight.id(),
            NodeWeight::ActionPrototype(weight) => weight.id(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.id(),
            NodeWeight::AttributeValue(weight) => weight.id(),
            NodeWeight::Category(weight) => weight.id(),
            NodeWeight::Component(weight) => weight.id(),
            NodeWeight::Content(weight) => weight.id(),
            NodeWeight::Func(weight) => weight.id(),
            NodeWeight::FuncArgument(weight) => weight.id(),
            NodeWeight::Ordering(weight) => weight.id(),
            NodeWeight::Prop(weight) => weight.id(),
            NodeWeight::Secret(weight) => weight.id(),
        }
    }

    pub fn increment_vector_clock(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Action(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::ActionPrototype(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::AttributePrototypeArgument(weight) => {
                weight.increment_vector_clock(change_set)
            }
            NodeWeight::AttributeValue(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Category(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Component(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Content(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Func(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::FuncArgument(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Ordering(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Prop(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Secret(weight) => weight.increment_vector_clock(change_set),
        }
    }

    pub fn lineage_id(&self) -> Ulid {
        match self {
            NodeWeight::Action(weight) => weight.lineage_id(),
            NodeWeight::ActionPrototype(weight) => weight.lineage_id(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.lineage_id(),
            NodeWeight::AttributeValue(weight) => weight.lineage_id(),
            NodeWeight::Category(weight) => weight.lineage_id(),
            NodeWeight::Component(weight) => weight.lineage_id(),
            NodeWeight::Content(weight) => weight.lineage_id(),
            NodeWeight::Func(weight) => weight.lineage_id(),
            NodeWeight::FuncArgument(weight) => weight.lineage_id(),
            NodeWeight::Ordering(weight) => weight.lineage_id(),
            NodeWeight::Prop(weight) => weight.lineage_id(),
            NodeWeight::Secret(weight) => weight.lineage_id(),
        }
    }

    pub fn mark_seen_at(&mut self, vector_clock_id: VectorClockId, seen_at: DateTime<Utc>) {
        match self {
            NodeWeight::Action(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::ActionPrototype(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::AttributePrototypeArgument(weight) => {
                weight.mark_seen_at(vector_clock_id, seen_at)
            }
            NodeWeight::AttributeValue(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Category(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Component(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Content(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Func(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::FuncArgument(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Ordering(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Prop(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Secret(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
        }
    }

    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSet,
        other: &NodeWeight,
    ) -> NodeWeightResult<()> {
        match (self, other) {
            (NodeWeight::Action(self_weight), NodeWeight::Action(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (
                NodeWeight::ActionPrototype(self_weight),
                NodeWeight::ActionPrototype(other_weight),
            ) => self_weight.merge_clocks(change_set, other_weight),
            (
                NodeWeight::AttributePrototypeArgument(self_weight),
                NodeWeight::AttributePrototypeArgument(other_weight),
            ) => self_weight.merge_clocks(change_set, other_weight),
            (NodeWeight::AttributeValue(self_weight), NodeWeight::AttributeValue(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::Category(self_weight), NodeWeight::Category(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::Component(self_weight), NodeWeight::Component(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::Content(self_weight), NodeWeight::Content(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::Func(self_weight), NodeWeight::Func(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::FuncArgument(self_weight), NodeWeight::FuncArgument(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::Ordering(self_weight), NodeWeight::Ordering(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::Prop(self_weight), NodeWeight::Prop(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::Secret(self_weight), NodeWeight::Secret(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            _ => Err(NodeWeightError::IncompatibleNodeWeightVariants),
        }
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        match self {
            NodeWeight::Action(weight) => weight.merkle_tree_hash(),
            NodeWeight::ActionPrototype(weight) => weight.merkle_tree_hash(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.merkle_tree_hash(),
            NodeWeight::AttributeValue(weight) => weight.merkle_tree_hash(),
            NodeWeight::Category(weight) => weight.merkle_tree_hash(),
            NodeWeight::Component(weight) => weight.merkle_tree_hash(),
            NodeWeight::Content(weight) => weight.merkle_tree_hash(),
            NodeWeight::Func(weight) => weight.merkle_tree_hash(),
            NodeWeight::FuncArgument(weight) => weight.merkle_tree_hash(),
            NodeWeight::Ordering(weight) => weight.merkle_tree_hash(),
            NodeWeight::Prop(weight) => weight.merkle_tree_hash(),
            NodeWeight::Secret(weight) => weight.merkle_tree_hash(),
        }
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Component(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Content(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Func(weight) => weight.new_content_hash(content_hash),
            NodeWeight::FuncArgument(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Prop(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Secret(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Action(_)
            | NodeWeight::ActionPrototype(_)
            | NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::Ordering(_) => Err(NodeWeightError::CannotSetContentHashOnKind),
        }
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSet,
    ) -> NodeWeightResult<Self> {
        let new_weight = match self {
            NodeWeight::Action(weight) => {
                NodeWeight::Action(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::ActionPrototype(weight) => {
                NodeWeight::ActionPrototype(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::AttributePrototypeArgument(weight) => {
                NodeWeight::AttributePrototypeArgument(
                    weight.new_with_incremented_vector_clock(change_set)?,
                )
            }
            NodeWeight::AttributeValue(weight) => {
                NodeWeight::AttributeValue(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Category(weight) => {
                NodeWeight::Category(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Component(weight) => {
                NodeWeight::Component(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Content(weight) => {
                NodeWeight::Content(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Func(weight) => {
                NodeWeight::Func(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::FuncArgument(weight) => {
                NodeWeight::FuncArgument(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Ordering(weight) => {
                NodeWeight::Ordering(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Prop(weight) => {
                NodeWeight::Prop(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Secret(weight) => {
                NodeWeight::Secret(weight.new_with_incremented_vector_clock(change_set)?)
            }
        };

        Ok(new_weight)
    }

    /// The node hash is used to compare nodes directly, and should be computed based on the data
    /// that is specific to the node weight, *and* the content_hash, so that changes are detected
    /// between nodes whether the content has changed or just the node weight has changed.
    pub fn node_hash(&self) -> ContentHash {
        match self {
            NodeWeight::Action(weight) => weight.node_hash(),
            NodeWeight::ActionPrototype(weight) => weight.node_hash(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.node_hash(),
            NodeWeight::AttributeValue(weight) => weight.node_hash(),
            NodeWeight::Category(weight) => weight.node_hash(),
            NodeWeight::Component(weight) => weight.node_hash(),
            NodeWeight::Content(weight) => weight.node_hash(),
            NodeWeight::Func(weight) => weight.node_hash(),
            NodeWeight::FuncArgument(weight) => weight.node_hash(),
            NodeWeight::Ordering(weight) => weight.node_hash(),
            NodeWeight::Prop(weight) => weight.node_hash(),
            NodeWeight::Secret(weight) => weight.node_hash(),
        }
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        match self {
            NodeWeight::Action(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::ActionPrototype(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::AttributePrototypeArgument(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::AttributeValue(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Category(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Component(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Content(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Func(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::FuncArgument(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Ordering(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Prop(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Secret(weight) => weight.set_merkle_tree_hash(new_hash),
        }
    }

    pub fn set_order(&mut self, change_set: &ChangeSet, order: Vec<Ulid>) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Ordering(ordering_weight) => ordering_weight.set_order(change_set, order),
            NodeWeight::Action(_)
            | NodeWeight::ActionPrototype(_)
            | NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::Component(_)
            | NodeWeight::Content(_)
            | NodeWeight::Func(_)
            | NodeWeight::FuncArgument(_)
            | NodeWeight::Prop(_)
            | NodeWeight::Secret(_) => Err(NodeWeightError::CannotSetOrderOnKind),
        }
    }

    pub fn set_vector_clock_recently_seen_to(
        &mut self,
        change_set: &ChangeSet,
        new_val: DateTime<Utc>,
    ) {
        match self {
            NodeWeight::Action(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::ActionPrototype(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::AttributePrototypeArgument(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::AttributeValue(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Category(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Component(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Content(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Func(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::FuncArgument(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Ordering(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Prop(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Secret(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
        }
    }

    pub fn vector_clock_first_seen(&self) -> &VectorClock {
        match self {
            NodeWeight::Action(weight) => weight.vector_clock_first_seen(),
            NodeWeight::ActionPrototype(weight) => weight.vector_clock_first_seen(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.vector_clock_first_seen(),
            NodeWeight::AttributeValue(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Category(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Component(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Content(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Func(weight) => weight.vector_clock_first_seen(),
            NodeWeight::FuncArgument(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Ordering(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Prop(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Secret(weight) => weight.vector_clock_first_seen(),
        }
    }

    pub fn vector_clock_recently_seen(&self) -> &VectorClock {
        match self {
            NodeWeight::Action(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::ActionPrototype(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::AttributeValue(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Category(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Component(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Content(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Func(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::FuncArgument(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Ordering(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Prop(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Secret(weight) => weight.vector_clock_recently_seen(),
        }
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        match self {
            NodeWeight::Action(weight) => weight.vector_clock_write(),
            NodeWeight::ActionPrototype(weight) => weight.vector_clock_write(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.vector_clock_write(),
            NodeWeight::AttributeValue(weight) => weight.vector_clock_write(),
            NodeWeight::Category(weight) => weight.vector_clock_write(),
            NodeWeight::Component(weight) => weight.vector_clock_write(),
            NodeWeight::Content(weight) => weight.vector_clock_write(),
            NodeWeight::Func(weight) => weight.vector_clock_write(),
            NodeWeight::FuncArgument(weight) => weight.vector_clock_write(),
            NodeWeight::Ordering(weight) => weight.vector_clock_write(),
            NodeWeight::Prop(weight) => weight.vector_clock_write(),
            NodeWeight::Secret(weight) => weight.vector_clock_write(),
        }
    }

    /// Many node kinds need to have complete control of their outgoing edges
    /// relative to another changeset in order to have a correctly constructed
    /// graph. For example, only one set of children of a given attribute value
    /// should "win" in a rebase operation, otherwise there could be duplicate
    /// child values for an attribute value. This method will be called during
    /// conflict detection in order to produce a conflict if the change set
    /// being rebased has unseen edges of this type for a given "container"
    /// node. If edge kinds are not returned here, those unseen edges will be
    /// silently merged with the `onto` changeset's edges. This a "business"
    /// logic problem, rather than a purely graph-theoretical one.
    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        match self {
            NodeWeight::Action(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::ActionPrototype(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::AttributePrototypeArgument(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::AttributeValue(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Category(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Component(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Content(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Func(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::FuncArgument(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Ordering(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Prop(weight) => weight.exclusive_outgoing_edges(),
            NodeWeight::Secret(weight) => weight.exclusive_outgoing_edges(),
        }
    }

    pub fn is_exclusive_outgoing_edge(&self, edge_kind: EdgeWeightKindDiscriminants) -> bool {
        self.exclusive_outgoing_edges().contains(&edge_kind)
    }

    pub fn get_action_node_weight(&self) -> NodeWeightResult<ActionNodeWeight> {
        match self {
            NodeWeight::Action(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Action,
                other.into(),
            )),
        }
    }

    pub fn get_action_prototype_node_weight(&self) -> NodeWeightResult<ActionPrototypeNodeWeight> {
        match self {
            NodeWeight::ActionPrototype(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::ActionPrototype,
                other.into(),
            )),
        }
    }

    pub fn get_attribute_prototype_argument_node_weight(
        &self,
    ) -> NodeWeightResult<AttributePrototypeArgumentNodeWeight> {
        match self {
            NodeWeight::AttributePrototypeArgument(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::AttributePrototypeArgument,
                other.into(),
            )),
        }
    }

    pub fn get_attribute_value_node_weight(&self) -> NodeWeightResult<AttributeValueNodeWeight> {
        match self {
            NodeWeight::AttributeValue(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::AttributeValue,
                other.into(),
            )),
        }
    }

    pub fn get_component_node_weight(&self) -> NodeWeightResult<ComponentNodeWeight> {
        match self {
            NodeWeight::Component(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Component,
                other.into(),
            )),
        }
    }

    pub fn get_prop_node_weight(&self) -> NodeWeightResult<PropNodeWeight> {
        match self {
            NodeWeight::Prop(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Prop,
                other.into(),
            )),
        }
    }

    pub fn get_func_node_weight(&self) -> NodeWeightResult<FuncNodeWeight> {
        match self {
            NodeWeight::Func(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Func,
                other.into(),
            )),
        }
    }

    pub fn get_func_argument_node_weight(&self) -> NodeWeightResult<FuncArgumentNodeWeight> {
        match self {
            NodeWeight::FuncArgument(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::FuncArgument,
                other.into(),
            )),
        }
    }

    pub fn get_secret_node_weight(&self) -> NodeWeightResult<SecretNodeWeight> {
        match self {
            NodeWeight::Secret(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Secret,
                other.into(),
            )),
        }
    }

    pub fn get_ordering_node_weight(&self) -> NodeWeightResult<OrderingNodeWeight> {
        match self {
            NodeWeight::Ordering(inner) => Ok(inner.to_owned()),
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Ordering,
                other.into(),
            )),
        }
    }

    pub fn get_content_node_weight_of_kind(
        &self,
        content_addr_discrim: ContentAddressDiscriminants,
    ) -> NodeWeightResult<ContentNodeWeight> {
        match self {
            NodeWeight::Content(inner) => {
                let inner_addr_discrim: ContentAddressDiscriminants =
                    inner.content_address().into();
                if inner_addr_discrim != content_addr_discrim {
                    return Err(NodeWeightError::UnexpectedContentAddressVariant(
                        content_addr_discrim,
                        inner_addr_discrim,
                    ));
                }

                Ok(inner.to_owned())
            }
            other => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                NodeWeightDiscriminants::Content,
                other.into(),
            )),
        }
    }

    pub fn get_option_content_node_weight_of_kind(
        &self,
        content_addr_discrim: ContentAddressDiscriminants,
    ) -> Option<ContentNodeWeight> {
        match self {
            NodeWeight::Content(inner) => {
                let inner_addr_discrim: ContentAddressDiscriminants =
                    inner.content_address().into();
                if inner_addr_discrim != content_addr_discrim {
                    return None;
                }
                Some(inner.to_owned())
            }
            _other => None,
        }
    }

    pub fn new_content(
        change_set: &ChangeSet,
        content_id: Ulid,
        kind: ContentAddress,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::Content(ContentNodeWeight::new(
            change_set, content_id, kind,
        )?))
    }

    pub fn new_action(
        change_set: &ChangeSet,
        originating_change_set_id: ChangeSetId,
        action_id: Ulid,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::Action(ActionNodeWeight::new(
            change_set,
            originating_change_set_id,
            action_id,
        )?))
    }

    pub fn new_action_prototype(
        change_set: &ChangeSet,
        action_id: Ulid,
        kind: ActionKind,
        name: String,
        description: Option<String>,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::ActionPrototype(ActionPrototypeNodeWeight::new(
            change_set,
            action_id,
            kind,
            name,
            description,
        )?))
    }

    pub fn new_attribute_value(
        change_set: &ChangeSet,
        attribute_value_id: Ulid,
        unprocessed_value: Option<ContentAddress>,
        value: Option<ContentAddress>,
        func_execution_pk: Option<FuncExecutionPk>,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::AttributeValue(AttributeValueNodeWeight::new(
            change_set,
            attribute_value_id,
            unprocessed_value,
            value,
            func_execution_pk,
        )?))
    }

    pub fn new_component(
        change_set: &ChangeSet,
        component_id: Ulid,
        content_hash: ContentHash,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::Component(ComponentNodeWeight::new(
            change_set,
            component_id,
            ContentAddress::Component(content_hash),
        )?))
    }

    pub fn new_prop(
        change_set: &ChangeSet,
        prop_id: Ulid,
        prop_kind: PropKind,
        name: impl AsRef<str>,
        content_hash: ContentHash,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::Prop(PropNodeWeight::new(
            change_set,
            prop_id,
            ContentAddress::Prop(content_hash),
            prop_kind,
            name.as_ref().to_string(),
        )?))
    }

    pub fn new_func(
        change_set: &ChangeSet,
        func_id: Ulid,
        name: impl AsRef<str>,
        func_kind: FuncKind,
        content_hash: ContentHash,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::Func(FuncNodeWeight::new(
            change_set,
            func_id,
            ContentAddress::Func(content_hash),
            name.as_ref().to_string(),
            func_kind,
        )?))
    }

    pub fn new_func_argument(
        change_set: &ChangeSet,
        func_arg_id: Ulid,
        name: impl AsRef<str>,
        content_hash: ContentHash,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::FuncArgument(FuncArgumentNodeWeight::new(
            change_set,
            func_arg_id,
            ContentAddress::FuncArg(content_hash),
            name.as_ref().to_string(),
        )?))
    }

    pub fn new_attribute_prototype_argument(
        change_set: &ChangeSet,
        attribute_prototype_argument_id: Ulid,
        targets: Option<ArgumentTargets>,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::AttributePrototypeArgument(
            AttributePrototypeArgumentNodeWeight::new(
                change_set,
                attribute_prototype_argument_id,
                targets,
            )?,
        ))
    }

    pub fn new_secret(
        change_set: &ChangeSet,
        secret_id: Ulid,
        encrypted_secret_key: EncryptedSecretKey,
        content_hash: ContentHash,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::Secret(SecretNodeWeight::new(
            change_set,
            secret_id,
            ContentAddress::Secret(content_hash),
            encrypted_secret_key,
        )?))
    }
}

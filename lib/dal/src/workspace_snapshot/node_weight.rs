use chrono::{DateTime, Utc};
use content_store::ContentHash;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use thiserror::Error;
use ulid::Ulid;

use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::FuncBackendKind;
use crate::{
    change_set_pointer::{ChangeSetPointer, ChangeSetPointerError},
    workspace_snapshot::{
        content_address::ContentAddress,
        vector_clock::{VectorClock, VectorClockError},
    },
    PropKind,
};

pub use attribute_prototype_argument_node_weight::AttributePrototypeArgumentNodeWeight;
pub use attribute_value_node_weight::AttributeValueNodeWeight;
pub use category_node_weight::CategoryNodeWeight;
pub use content_node_weight::ContentNodeWeight;
pub use func_argument_node_weight::FuncArgumentNodeWeight;
pub use func_node_weight::FuncNodeWeight;
pub use ordering_node_weight::OrderingNodeWeight;
pub use prop_node_weight::PropNodeWeight;

use self::attribute_prototype_argument_node_weight::ArgumentTargets;

use super::content_address::ContentAddressDiscriminants;

pub mod attribute_prototype_argument_node_weight;
pub mod attribute_value_node_weight;
pub mod category_node_weight;
pub mod content_node_weight;
pub mod func_argument_node_weight;
pub mod func_node_weight;
pub mod ordering_node_weight;
pub mod prop_node_weight;

#[derive(Debug, Error)]
pub enum NodeWeightError {
    #[error("Cannot set content hash directly on node weight kind")]
    CannotSetContentHashOnKind,
    #[error("Cannot set content order directly on node weight kind")]
    CannotSetOrderOnKind,
    #[error("Cannot update root node's content hash")]
    CannotUpdateRootNodeContentHash,
    #[error("ChangeSet error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("Incompatible node weights")]
    IncompatibleNodeWeightVariants,
    #[error("Invalid ContentAddress variant ({0}) for NodeWeight variant ({1})")]
    InvalidContentAddressForWeightKind(String, String),
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
    AttributePrototypeArgument(AttributePrototypeArgumentNodeWeight),
    AttributeValue(AttributeValueNodeWeight),
    Category(CategoryNodeWeight),
    Content(ContentNodeWeight),
    Func(FuncNodeWeight),
    FuncArgument(FuncArgumentNodeWeight),
    Ordering(OrderingNodeWeight),
    Prop(PropNodeWeight),
}

impl NodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => weight.content_hash(),
            NodeWeight::AttributeValue(weight) => weight.content_hash(),
            NodeWeight::Category(weight) => weight.content_hash(),
            NodeWeight::Content(weight) => weight.content_hash(),
            NodeWeight::Func(weight) => weight.content_hash(),
            NodeWeight::FuncArgument(weight) => weight.content_hash(),
            NodeWeight::Ordering(weight) => weight.content_hash(),
            NodeWeight::Prop(weight) => weight.content_hash(),
        }
    }

    pub fn content_address_discriminants(&self) -> Option<ContentAddressDiscriminants> {
        match self {
            NodeWeight::Content(weight) => Some(weight.content_address().into()),
            NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::Func(_)
            | NodeWeight::FuncArgument(_)
            | NodeWeight::Ordering(_)
            | NodeWeight::Prop(_) => None,
        }
    }

    pub fn id(&self) -> Ulid {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => weight.id(),
            NodeWeight::AttributeValue(weight) => weight.id(),
            NodeWeight::Category(weight) => weight.id(),
            NodeWeight::Content(weight) => weight.id(),
            NodeWeight::Func(weight) => weight.id(),
            NodeWeight::FuncArgument(weight) => weight.id(),
            NodeWeight::Ordering(weight) => weight.id(),
            NodeWeight::Prop(weight) => weight.id(),
        }
    }

    pub fn increment_vector_clock(
        &mut self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<()> {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => {
                weight.increment_vector_clock(change_set)
            }
            NodeWeight::AttributeValue(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Category(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Content(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Func(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::FuncArgument(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Ordering(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Prop(weight) => weight.increment_vector_clock(change_set),
        }
    }

    pub fn lineage_id(&self) -> Ulid {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => weight.lineage_id(),
            NodeWeight::AttributeValue(weight) => weight.lineage_id(),
            NodeWeight::Category(weight) => weight.lineage_id(),
            NodeWeight::Content(weight) => weight.lineage_id(),
            NodeWeight::Func(weight) => weight.lineage_id(),
            NodeWeight::FuncArgument(weight) => weight.lineage_id(),
            NodeWeight::Ordering(weight) => weight.lineage_id(),
            NodeWeight::Prop(weight) => weight.lineage_id(),
        }
    }

    pub fn mark_seen_at(&mut self, vector_clock_id: VectorClockId, seen_at: DateTime<Utc>) {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => {
                weight.mark_seen_at(vector_clock_id, seen_at)
            }
            NodeWeight::AttributeValue(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Category(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Content(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Func(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::FuncArgument(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Ordering(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Prop(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
        }
    }

    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSetPointer,
        other: &NodeWeight,
    ) -> NodeWeightResult<()> {
        match (self, other) {
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
            _ => Err(NodeWeightError::IncompatibleNodeWeightVariants),
        }
    }

    pub fn merkle_tree_hash(&self) -> ContentHash {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => weight.merkle_tree_hash(),
            NodeWeight::AttributeValue(weight) => weight.merkle_tree_hash(),
            NodeWeight::Category(weight) => weight.merkle_tree_hash(),
            NodeWeight::Content(weight) => weight.merkle_tree_hash(),
            NodeWeight::Func(weight) => weight.merkle_tree_hash(),
            NodeWeight::FuncArgument(weight) => weight.merkle_tree_hash(),
            NodeWeight::Ordering(weight) => weight.merkle_tree_hash(),
            NodeWeight::Prop(weight) => weight.merkle_tree_hash(),
        }
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Content(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Func(weight) => weight.new_content_hash(content_hash),
            NodeWeight::FuncArgument(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Prop(weight) => weight.new_content_hash(content_hash),
            NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::Ordering(_) => Err(NodeWeightError::CannotSetContentHashOnKind),
        }
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<Self> {
        let new_weight = match self {
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
        };

        Ok(new_weight)
    }

    /// The node hash is used to compare nodes directly, and should be computed based on the data
    /// that is specific to the node weight, *and* the content_hash, so that changes are detected
    /// between nodes whether the content has changed or just the node weight has changed.
    pub fn node_hash(&self) -> ContentHash {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => weight.node_hash(),
            NodeWeight::AttributeValue(weight) => weight.node_hash(),
            NodeWeight::Category(weight) => weight.node_hash(),
            NodeWeight::Content(weight) => weight.node_hash(),
            NodeWeight::Func(weight) => weight.node_hash(),
            NodeWeight::FuncArgument(weight) => weight.node_hash(),
            NodeWeight::Ordering(weight) => weight.node_hash(),
            NodeWeight::Prop(weight) => weight.node_hash(),
        }
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::AttributeValue(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Category(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Content(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Func(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::FuncArgument(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Ordering(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Prop(weight) => weight.set_merkle_tree_hash(new_hash),
        }
    }

    pub fn set_order(
        &mut self,
        change_set: &ChangeSetPointer,
        order: Vec<Ulid>,
    ) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Ordering(ordering_weight) => ordering_weight.set_order(change_set, order),

            NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::Content(_)
            | NodeWeight::Func(_)
            | NodeWeight::FuncArgument(_)
            | NodeWeight::Prop(_) => Err(NodeWeightError::CannotSetOrderOnKind),
        }
    }

    pub fn set_vector_clock_recently_seen_to(
        &mut self,
        change_set: &ChangeSetPointer,
        new_val: DateTime<Utc>,
    ) {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::AttributeValue(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Category(weight) => {
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
        }
    }

    pub fn vector_clock_first_seen(&self) -> &VectorClock {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => weight.vector_clock_first_seen(),
            NodeWeight::AttributeValue(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Category(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Content(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Func(weight) => weight.vector_clock_first_seen(),
            NodeWeight::FuncArgument(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Ordering(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Prop(weight) => weight.vector_clock_first_seen(),
        }
    }

    pub fn vector_clock_recently_seen(&self) -> &VectorClock {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::AttributeValue(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Category(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Content(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Func(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::FuncArgument(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Ordering(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Prop(weight) => weight.vector_clock_recently_seen(),
        }
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        match self {
            NodeWeight::AttributePrototypeArgument(weight) => weight.vector_clock_write(),
            NodeWeight::AttributeValue(weight) => weight.vector_clock_write(),
            NodeWeight::Category(weight) => weight.vector_clock_write(),
            NodeWeight::Content(weight) => weight.vector_clock_write(),
            NodeWeight::Func(weight) => weight.vector_clock_write(),
            NodeWeight::FuncArgument(weight) => weight.vector_clock_write(),
            NodeWeight::Ordering(weight) => weight.vector_clock_write(),
            NodeWeight::Prop(weight) => weight.vector_clock_write(),
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
        change_set: &ChangeSetPointer,
        content_id: Ulid,
        kind: ContentAddress,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::Content(ContentNodeWeight::new(
            change_set, content_id, kind,
        )?))
    }

    pub fn new_attribute_value(
        change_set: &ChangeSetPointer,
        attribute_value_id: Ulid,
        unprocessed_value: Option<ContentAddress>,
        value: Option<ContentAddress>,
        materialized_view: Option<ContentAddress>,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::AttributeValue(AttributeValueNodeWeight::new(
            change_set,
            attribute_value_id,
            unprocessed_value,
            value,
            materialized_view,
        )?))
    }

    pub fn new_prop(
        change_set: &ChangeSetPointer,
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
        change_set: &ChangeSetPointer,
        func_id: Ulid,
        name: impl AsRef<str>,
        backend_kind: FuncBackendKind,
        content_hash: ContentHash,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::Func(FuncNodeWeight::new(
            change_set,
            func_id,
            ContentAddress::Func(content_hash),
            name.as_ref().to_string(),
            backend_kind,
        )?))
    }

    pub fn new_func_argument(
        change_set: &ChangeSetPointer,
        func_arg_id: Ulid,
        name: impl AsRef<str>,
        content_hash: ContentHash,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::FuncArgument(FuncArgumentNodeWeight::new(
            change_set,
            func_arg_id,
            ContentAddress::Func(content_hash),
            name.as_ref().to_string(),
        )?))
    }

    pub fn new_attribute_prototype_argument(
        change_set: &ChangeSetPointer,
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
}

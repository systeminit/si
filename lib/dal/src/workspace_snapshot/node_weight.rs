use chrono::{DateTime, Utc};
use content_store::ContentHash;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use thiserror::Error;
use ulid::Ulid;

use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
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

pub use category_node_weight::CategoryNodeWeight;
pub use content_node_weight::ContentNodeWeight;
pub use func_node_weight::FuncNodeWeight;
pub use ordering_node_weight::OrderingNodeWeight;
pub use prop_node_weight::PropNodeWeight;

use super::content_address::ContentAddressDiscriminants;

pub mod category_node_weight;
pub mod content_node_weight;
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
    #[error("Unexpected node weight variant: {1} expected {0}")]
    UnexpectedNodeWeightVariant(NodeWeightDiscriminants, NodeWeightDiscriminants),
    #[error("Vector Clock error: {0}")]
    VectorClock(#[from] VectorClockError),
}

pub type NodeWeightResult<T> = Result<T, NodeWeightError>;

#[derive(Debug, Serialize, Deserialize, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize))]
pub enum NodeWeight {
    Category(CategoryNodeWeight),
    Content(ContentNodeWeight),
    Func(FuncNodeWeight),
    Ordering(OrderingNodeWeight),
    Prop(PropNodeWeight),
}

impl NodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            NodeWeight::Category(weight) => weight.content_hash(),
            NodeWeight::Content(weight) => weight.content_hash(),
            NodeWeight::Func(weight) => weight.content_hash(),
            NodeWeight::Ordering(weight) => weight.content_hash(),
            NodeWeight::Prop(weight) => weight.content_hash(),
        }
    }

    pub fn content_address_discriminants(&self) -> Option<ContentAddressDiscriminants> {
        match self {
            NodeWeight::Content(weight) => Some(weight.content_address().into()),
            NodeWeight::Category(_)
            | NodeWeight::Func(_)
            | NodeWeight::Ordering(_)
            | NodeWeight::Prop(_) => None,
        }
    }

    /// Get the label for debugging via the raw
    /// [`ContentAddressDiscriminant`](ContentAddressDiscriminants), the raw
    /// [`NodeWeightDiscriminant`](NodeWeightDiscriminants) or other data from the the
    /// [`NodeWeight`].
    #[allow(dead_code)]
    pub fn debugging_label(&self) -> String {
        match self {
            NodeWeight::Content(weight) => {
                ContentAddressDiscriminants::from(weight.content_address()).to_string()
            }
            NodeWeight::Category(category_node_weight) => match category_node_weight.kind() {
                CategoryNodeKind::Component => "Components (Category)".to_string(),
                CategoryNodeKind::Func => "Funcs (Category)".to_string(),
                CategoryNodeKind::Schema => "Schemas (Category)".to_string(),
            },
            NodeWeight::Func(func_node_weight) => format!("Func\n{}", func_node_weight.name()),
            NodeWeight::Ordering(_) => NodeWeightDiscriminants::Ordering.to_string(),
            NodeWeight::Prop(prop_node_weight) => format!("Prop\n{}", prop_node_weight.name()),
        }
    }

    pub fn id(&self) -> Ulid {
        match self {
            NodeWeight::Category(weight) => weight.id(),
            NodeWeight::Content(weight) => weight.id(),
            NodeWeight::Func(weight) => weight.id(),
            NodeWeight::Ordering(weight) => weight.id(),
            NodeWeight::Prop(weight) => weight.id(),
        }
    }

    pub fn increment_vector_clock(
        &mut self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Category(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Content(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Func(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Ordering(weight) => weight.increment_vector_clock(change_set),
            NodeWeight::Prop(weight) => weight.increment_vector_clock(change_set),
        }
    }

    pub fn lineage_id(&self) -> Ulid {
        match self {
            NodeWeight::Category(weight) => weight.lineage_id(),
            NodeWeight::Content(weight) => weight.lineage_id(),
            NodeWeight::Func(weight) => weight.lineage_id(),
            NodeWeight::Ordering(weight) => weight.lineage_id(),
            NodeWeight::Prop(weight) => weight.lineage_id(),
        }
    }

    pub fn mark_seen_at(&mut self, vector_clock_id: VectorClockId, seen_at: DateTime<Utc>) {
        match self {
            NodeWeight::Category(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Content(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
            NodeWeight::Func(weight) => weight.mark_seen_at(vector_clock_id, seen_at),
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
            (NodeWeight::Category(self_weight), NodeWeight::Category(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::Content(self_weight), NodeWeight::Content(other_weight)) => {
                self_weight.merge_clocks(change_set, other_weight)
            }
            (NodeWeight::Func(self_weight), NodeWeight::Func(other_weight)) => {
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
            NodeWeight::Category(weight) => weight.merkle_tree_hash(),
            NodeWeight::Content(weight) => weight.merkle_tree_hash(),
            NodeWeight::Func(weight) => weight.merkle_tree_hash(),
            NodeWeight::Ordering(weight) => weight.merkle_tree_hash(),
            NodeWeight::Prop(weight) => weight.merkle_tree_hash(),
        }
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Category(_) => Err(NodeWeightError::CannotSetContentHashOnKind),
            NodeWeight::Content(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Func(weight) => weight.new_content_hash(content_hash),
            NodeWeight::Ordering(_) => Err(NodeWeightError::CannotSetContentHashOnKind),
            NodeWeight::Prop(weight) => weight.new_content_hash(content_hash),
        }
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<Self> {
        let new_weight = match self {
            NodeWeight::Category(weight) => {
                NodeWeight::Category(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Content(weight) => {
                NodeWeight::Content(weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Func(weight) => {
                NodeWeight::Func(weight.new_with_incremented_vector_clock(change_set)?)
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

    pub fn node_hash(&self) -> ContentHash {
        match self {
            NodeWeight::Category(weight) => weight.node_hash(),
            NodeWeight::Content(weight) => weight.node_hash(),
            NodeWeight::Func(weight) => weight.node_hash(),
            NodeWeight::Ordering(weight) => weight.node_hash(),
            NodeWeight::Prop(weight) => weight.node_hash(),
        }
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        match self {
            NodeWeight::Category(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Content(weight) => weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Func(weight) => weight.set_merkle_tree_hash(new_hash),
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
            NodeWeight::Category(_)
            | NodeWeight::Content(_)
            | NodeWeight::Func(_)
            | NodeWeight::Prop(_) => Err(NodeWeightError::CannotSetOrderOnKind),
        }
    }

    pub fn set_vector_clock_recently_seen_to(
        &mut self,
        change_set: &ChangeSetPointer,
        new_val: DateTime<Utc>,
    ) {
        match self {
            NodeWeight::Category(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Content(weight) => {
                weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Func(weight) => {
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
            NodeWeight::Category(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Content(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Func(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Ordering(weight) => weight.vector_clock_first_seen(),
            NodeWeight::Prop(weight) => weight.vector_clock_first_seen(),
        }
    }

    pub fn vector_clock_recently_seen(&self) -> &VectorClock {
        match self {
            NodeWeight::Category(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Content(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Func(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Ordering(weight) => weight.vector_clock_recently_seen(),
            NodeWeight::Prop(weight) => weight.vector_clock_recently_seen(),
        }
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        match self {
            NodeWeight::Category(weight) => weight.vector_clock_write(),
            NodeWeight::Content(weight) => weight.vector_clock_write(),
            NodeWeight::Func(weight) => weight.vector_clock_write(),
            NodeWeight::Ordering(weight) => weight.vector_clock_write(),
            NodeWeight::Prop(weight) => weight.vector_clock_write(),
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

    // NOTE(nick): individual node weight funcs below.

    pub fn new_content(
        change_set: &ChangeSetPointer,
        content_id: Ulid,
        kind: ContentAddress,
    ) -> NodeWeightResult<Self> {
        Ok(NodeWeight::Content(ContentNodeWeight::new(
            change_set, content_id, kind,
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
}

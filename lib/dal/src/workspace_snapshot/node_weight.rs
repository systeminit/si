use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

use crate::{
    change_set_pointer::{ChangeSetPointer, ChangeSetPointerError},
    workspace_snapshot::{
        content_address::ContentAddress,
        vector_clock::{VectorClock, VectorClockError},
    },
    ContentHash, PropKind,
};

pub use content_node_weight::ContentNodeWeight;
pub use ordering_node_weight::OrderingNodeWeight;
pub use prop_node_weight::PropNodeWeight;

pub mod content_node_weight;
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
    #[error("Vector Clock error: {0}")]
    VectorClock(#[from] VectorClockError),
}

pub type NodeWeightResult<T> = Result<T, NodeWeightError>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeWeight {
    Content(ContentNodeWeight),
    Ordering(OrderingNodeWeight),
    Prop(PropNodeWeight),
}

impl NodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        match self {
            NodeWeight::Content(content_weight) => content_weight.content_hash(),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.content_hash(),
            NodeWeight::Prop(prop_weight) => prop_weight.content_hash(),
        }
    }

    pub fn id(&self) -> Ulid {
        match self {
            NodeWeight::Content(content_weight) => content_weight.id(),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.id(),
            NodeWeight::Prop(prop_weight) => prop_weight.id(),
        }
    }

    pub fn increment_vector_clock(
        &mut self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Content(content_weight) => {
                content_weight.increment_vector_clock(change_set)
            }
            NodeWeight::Ordering(ordering_weight) => {
                ordering_weight.increment_vector_clock(change_set)
            }
            NodeWeight::Prop(prop_weight) => prop_weight.increment_vector_clock(change_set),
        }
    }

    pub fn lineage_id(&self) -> Ulid {
        match self {
            NodeWeight::Content(content_weight) => content_weight.lineage_id(),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.lineage_id(),
            NodeWeight::Prop(prop_weight) => prop_weight.lineage_id(),
        }
    }

    pub fn mark_seen_at(&mut self, change_set: &ChangeSetPointer, seen_at: DateTime<Utc>) {
        match self {
            NodeWeight::Content(content_weight) => content_weight.mark_seen_at(change_set, seen_at),
            NodeWeight::Ordering(ordering_weight) => {
                ordering_weight.mark_seen_at(change_set, seen_at)
            }
            NodeWeight::Prop(prop_weight) => prop_weight.mark_seen_at(change_set, seen_at),
        }
    }

    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSetPointer,
        other: &NodeWeight,
    ) -> NodeWeightResult<()> {
        match (self, other) {
            (
                NodeWeight::Content(self_content_weight),
                NodeWeight::Content(other_content_weight),
            ) => self_content_weight.merge_clocks(change_set, other_content_weight),
            (
                NodeWeight::Ordering(self_ordering_weight),
                NodeWeight::Ordering(other_ordering_weight),
            ) => self_ordering_weight.merge_clocks(change_set, other_ordering_weight),
            (NodeWeight::Prop(self_prop_weight), NodeWeight::Prop(other_prop_weight)) => {
                self_prop_weight.merge_clocks(change_set, other_prop_weight)
            }
            _ => Err(NodeWeightError::IncompatibleNodeWeightVariants),
        }
    }

    pub fn merkle_tree_hash(&self) -> ContentHash {
        match self {
            NodeWeight::Content(content_weight) => content_weight.merkle_tree_hash(),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.merkle_tree_hash(),
            NodeWeight::Prop(prop_weight) => prop_weight.merkle_tree_hash(),
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

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Content(content_weight) => content_weight.new_content_hash(content_hash),
            NodeWeight::Ordering(_) => Err(NodeWeightError::CannotSetContentHashOnKind),
            NodeWeight::Prop(prop_weight) => prop_weight.new_content_hash(content_hash),
        }
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

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<Self> {
        let new_weight = match self {
            NodeWeight::Content(content_weight) => {
                NodeWeight::Content(content_weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Ordering(ordering_weight) => {
                NodeWeight::Ordering(ordering_weight.new_with_incremented_vector_clock(change_set)?)
            }
            NodeWeight::Prop(prop_weight) => {
                NodeWeight::Prop(prop_weight.new_with_incremented_vector_clock(change_set)?)
            }
        };

        Ok(new_weight)
    }

    pub fn node_hash(&self) -> ContentHash {
        match self {
            NodeWeight::Content(content_weight) => content_weight.node_hash(),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.node_hash(),
            NodeWeight::Prop(prop_weight) => prop_weight.node_hash(),
        }
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        match self {
            NodeWeight::Content(content_weight) => content_weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.set_merkle_tree_hash(new_hash),
            NodeWeight::Prop(prop_weight) => prop_weight.set_merkle_tree_hash(new_hash),
        }
    }

    pub fn set_order(
        &mut self,
        change_set: &ChangeSetPointer,
        order: Vec<Ulid>,
    ) -> NodeWeightResult<()> {
        match self {
            NodeWeight::Content(_) => Err(NodeWeightError::CannotSetOrderOnKind),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.set_order(change_set, order),
            NodeWeight::Prop(_) => Err(NodeWeightError::CannotSetOrderOnKind),
        }
    }

    pub fn set_vector_clock_recently_seen_to(
        &mut self,
        change_set: &ChangeSetPointer,
        new_val: DateTime<Utc>,
    ) {
        match self {
            NodeWeight::Content(content_weight) => {
                content_weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Ordering(ordering_weight) => {
                ordering_weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
            NodeWeight::Prop(prop_weight) => {
                prop_weight.set_vector_clock_recently_seen_to(change_set, new_val)
            }
        }
    }

    pub fn vector_clock_first_seen(&self) -> &VectorClock {
        match self {
            NodeWeight::Content(content_weight) => content_weight.vector_clock_first_seen(),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.vector_clock_first_seen(),
            NodeWeight::Prop(prop_weight) => prop_weight.vector_clock_first_seen(),
        }
    }

    pub fn vector_clock_recently_seen(&self) -> &VectorClock {
        match self {
            NodeWeight::Content(content_weight) => content_weight.vector_clock_recently_seen(),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.vector_clock_recently_seen(),
            NodeWeight::Prop(prop_weight) => prop_weight.vector_clock_recently_seen(),
        }
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        match self {
            NodeWeight::Content(content_weight) => content_weight.vector_clock_write(),
            NodeWeight::Ordering(ordering_weight) => ordering_weight.vector_clock_write(),
            NodeWeight::Prop(prop_weight) => prop_weight.vector_clock_write(),
        }
    }
}

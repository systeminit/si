//! Nodes

use crate::workspace_snapshot::{
    change_set::{ChangeSet, ChangeSetError},
    vector_clock::{VectorClock, VectorClockError},
    ContentHash,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

#[derive(Debug, Error)]
pub enum NodeWeightError {
    #[error("ChangeSet error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("Vector Clock error: {0}")]
    VectorClock(#[from] VectorClockError),
}

pub type NodeWeightResult<T> = Result<T, NodeWeightError>;

pub type OriginId = Ulid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
/// The type of the object, and the content-addressable-storage address (content hash)
/// of the object itself.
pub enum NodeWeightKind {
    Root,
    SchemaVariant(ContentHash),
    Schema(ContentHash),
    Component(ContentHash),
    Func(ContentHash),
    Prop(ContentHash),
}

impl NodeWeightKind {
    fn content_hash(&self) -> ContentHash {
        match self {
            NodeWeightKind::Root => None,
            NodeWeightKind::SchemaVariant(id) => Some(*id),
            NodeWeightKind::Schema(id) => Some(*id),
            NodeWeightKind::Component(id) => Some(*id),
            NodeWeightKind::Func(id) => Some(*id),
            NodeWeightKind::Prop(id) => Some(*id),
        }
        .unwrap_or_default()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeWeight {
    /// The stable local ID of the object in question. Mainly used by external things like
    /// the UI to be able to say "do X to _this_ thing" since the `NodeIndex` is an
    /// internal implementation detail, and the content ID wrapped by the
    /// [`NodeWeightKind`] changes whenever something about the node itself changes (for
    /// example, the name, or type of a [`Prop`].)
    pub id: Ulid,
    /// Globally stable ID for tracking the "lineage" of a thing to determine whether it
    /// should be trying to receive updates.
    pub origin_id: OriginId,
    /// What type of thing is this node representing, and what is the content hash used to
    /// retrieve the data for this specific node.
    pub kind: NodeWeightKind,
    /// [Merkle tree](https://en.wikipedia.org/wiki/Merkle_tree) hash for the graph
    /// starting with this node as the root. Mainly useful in quickly determining "has
    /// something changed anywhere in this (sub)graph".
    pub merkle_tree_hash: ContentHash,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_clock_seen: Option<VectorClock>,
    pub vector_clock_write: VectorClock,
}

impl NodeWeight {
    pub fn new(change_set: &ChangeSet, kind: NodeWeightKind) -> NodeWeightResult<NodeWeight> {
        Ok(NodeWeight {
            id: change_set.generate_ulid()?,
            origin_id: change_set.generate_ulid()?.into(),
            kind,
            merkle_tree_hash: ContentHash::default(),
            vector_clock_seen: None,
            vector_clock_write: VectorClock::new(change_set)?,
        })
    }

    pub fn content_hash(&self) -> ContentHash {
        self.kind.content_hash()
    }

    pub fn merkle_tree_hash(&self) -> ContentHash {
        self.merkle_tree_hash
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn modify(&self, change_set: &ChangeSet) -> NodeWeightResult<Self> {
        let vector_clock_seen = match &self.vector_clock_seen {
            Some(vcs) => {
                let mut new_vcs = vcs.clone();
                new_vcs.inc(change_set)?;

                Some(new_vcs)
            }
            None => None,
        };
        let mut vector_clock_write = self.vector_clock_write.clone();
        vector_clock_write.inc(change_set)?;

        Ok(NodeWeight {
            id: self.id,
            origin_id: self.origin_id,
            kind: self.kind,
            merkle_tree_hash: self.merkle_tree_hash,
            vector_clock_seen,
            vector_clock_write,
        })
    }
}

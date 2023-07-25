//! Nodes

use crate::{workspace_snapshot::{
    change_set::{ChangeSet, ChangeSetError},
    vector_clock::{VectorClock, VectorClockError},
    ContentHash,
}, PropKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

#[derive(Debug, Error)]
pub enum NodeWeightError {
    #[error("Cannot update root node's content hash")]
    CannotUpdateRootNodeContentHash,
    #[error("ChangeSet error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("No Seen Vector Clock available")]
    NoSeenVectorClock,
    #[error("Vector Clock error: {0}")]
    VectorClock(#[from] VectorClockError),
}

pub type NodeWeightResult<T> = Result<T, NodeWeightError>;

pub type OriginId = Ulid;

#[remain::sorted]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
/// The type of the object, and the content-addressable-storage address (content hash)
/// of the object itself.
pub enum NodeWeightKind {
    Component(ContentHash),
    Func(ContentHash),
    FuncArg(ContentHash),
    Prop(ContentHash),
    Root,
    Schema(ContentHash),
    SchemaVariant(ContentHash),
}

impl NodeWeightKind {
    fn content_hash(&self) -> ContentHash {
        match self {
            NodeWeightKind::Component(id) => Some(*id),
            NodeWeightKind::Func(id) => Some(*id),
            NodeWeightKind::FuncArg(id) => Some(*id),
            NodeWeightKind::Prop(id) => Some(*id),
            NodeWeightKind::Root => None,
            NodeWeightKind::Schema(id) => Some(*id),
            NodeWeightKind::SchemaVariant(id) => Some(*id),
        }
        .unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Clone)]
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
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub container_content_order: Option<Vec<Ulid>>,
    /// [Merkle tree](https://en.wikipedia.org/wiki/Merkle_tree) hash for the graph
    /// starting with this node as the root. Mainly useful in quickly determining "has
    /// something changed anywhere in this (sub)graph".
    pub merkle_tree_hash: ContentHash,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_clock_seen: Option<VectorClock>,
    pub vector_clock_write: VectorClock,
}

impl NodeWeight {
    pub fn new(
        change_set: &ChangeSet,
        id: Ulid,
        kind: NodeWeightKind,
    ) -> NodeWeightResult<NodeWeight> {
        Ok(NodeWeight {
            id,
            origin_id: change_set.generate_ulid()?,
            kind,
            container_content_order: None,
            merkle_tree_hash: ContentHash::default(),
            vector_clock_seen: None,
            vector_clock_write: VectorClock::new(change_set)?,
        })
    }

    pub fn new_with_seen_vector_clock(
        change_set: &ChangeSet,
        kind: NodeWeightKind,
    ) -> NodeWeightResult<NodeWeight> {
        Ok(NodeWeight {
            id: change_set.generate_ulid()?,
            origin_id: change_set.generate_ulid()?,
            kind,
            container_content_order: Default::default(),
            merkle_tree_hash: ContentHash::default(),
            vector_clock_seen: Some(VectorClock::new(change_set)?),
            vector_clock_write: VectorClock::new(change_set)?,
        })
    }

    pub fn content_hash(&self) -> ContentHash {
        self.kind.content_hash()
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_kind = match &self.kind {
            NodeWeightKind::Component(_) => NodeWeightKind::Component(content_hash),
            NodeWeightKind::Func(_) => NodeWeightKind::Func(content_hash),
            NodeWeightKind::FuncArg(_) => NodeWeightKind::FuncArg(content_hash),
            NodeWeightKind::Prop(_) => NodeWeightKind::Prop(content_hash),
            NodeWeightKind::Root => return Err(NodeWeightError::CannotUpdateRootNodeContentHash),
            NodeWeightKind::Schema(_) => NodeWeightKind::Schema(content_hash),
            NodeWeightKind::SchemaVariant(_) => NodeWeightKind::SchemaVariant(content_hash),
        };

        self.kind = new_kind;

        Ok(())
    }

    pub fn merkle_tree_hash(&self) -> ContentHash {
        self.merkle_tree_hash
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn increment_vector_clocks(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        if let Some(vcs) = &mut self.vector_clock_seen {
            vcs.inc(change_set)?;
        };
        self.vector_clock_write.inc(change_set)?;

        Ok(())
    }

    pub fn increment_seen_vector_clock(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        if let Some(vcs) = &mut self.vector_clock_seen {
            vcs.inc(change_set)?;

            Ok(())
        } else {
            Err(NodeWeightError::NoSeenVectorClock)
        }
    }

    pub fn new_with_incremented_vector_clocks(
        &self,
        change_set: &ChangeSet,
    ) -> NodeWeightResult<Self> {
        let mut new_node_weight = self.clone();
        new_node_weight.increment_vector_clocks(change_set)?;

        Ok(new_node_weight)
    }

    pub fn merge_clocks(&mut self, change_set: &ChangeSet, other: &NodeWeight) -> NodeWeightResult<()> {
        self.vector_clock_write.merge(change_set, &other.vector_clock_write)?;

        if let Some(local_vector_clock_seen) = &mut self.vector_clock_seen {
            if let Some(remote_vector_clock_seen) = &other.vector_clock_seen {
                local_vector_clock_seen.merge(change_set, remote_vector_clock_seen)?;

                Ok(())
            } else {
                Err(NodeWeightError::NoSeenVectorClock)
            }
        } else {
            Err(NodeWeightError::NoSeenVectorClock)
        }
    }
}

impl std::fmt::Debug for NodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NodeWeight")
            .field("id", &self.id.to_string())
            .field("origin_id", &self.origin_id.to_string())
            .field("kind", &self.kind)
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .field("vector_clock_seen", &self.vector_clock_seen)
            .field("vector_clock_write", &self.vector_clock_write)
            .finish()
    }
}

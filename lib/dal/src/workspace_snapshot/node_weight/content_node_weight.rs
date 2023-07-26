use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::workspace_snapshot::{
    change_set::ChangeSet,
    content_hash::ContentHash,
    node_weight::{NodeWeightError, NodeWeightResult},
    vector_clock::VectorClock,
};

pub type OriginId = Ulid;

#[remain::sorted]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
/// The type of the object, and the content-addressable-storage address (content hash)
/// of the object itself.
pub enum ContentKind {
    Component(ContentHash),
    Func(ContentHash),
    FuncArg(ContentHash),
    Prop(ContentHash),
    Root,
    Schema(ContentHash),
    SchemaVariant(ContentHash),
}

impl ContentKind {
    fn content_hash(&self) -> ContentHash {
        match self {
            ContentKind::Component(id) => Some(*id),
            ContentKind::Func(id) => Some(*id),
            ContentKind::FuncArg(id) => Some(*id),
            ContentKind::Prop(id) => Some(*id),
            ContentKind::Root => None,
            ContentKind::Schema(id) => Some(*id),
            ContentKind::SchemaVariant(id) => Some(*id),
        }
        .unwrap_or_default()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ContentNodeWeight {
    /// The stable local ID of the object in question. Mainly used by external things like
    /// the UI to be able to say "do X to _this_ thing" since the `NodeIndex` is an
    /// internal implementation detail, and the content ID wrapped by the
    /// [`NodeWeightKind`] changes whenever something about the node itself changes (for
    /// example, the name, or type of a [`Prop`].)
    id: Ulid,
    /// Globally stable ID for tracking the "lineage" of a thing to determine whether it
    /// should be trying to receive updates.
    origin_id: OriginId,
    /// What type of thing is this node representing, and what is the content hash used to
    /// retrieve the data for this specific node.
    kind: ContentKind,
    /// [Merkle tree](https://en.wikipedia.org/wiki/Merkle_tree) hash for the graph
    /// starting with this node as the root. Mainly useful in quickly determining "has
    /// something changed anywhere in this (sub)graph".
    merkle_tree_hash: ContentHash,
    #[serde(skip_serializing_if = "Option::is_none")]
    vector_clock_seen: Option<VectorClock>,
    vector_clock_write: VectorClock,
}

impl ContentNodeWeight {
    pub fn new(change_set: &ChangeSet, id: Ulid, kind: ContentKind) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            origin_id: change_set.generate_ulid()?,
            kind,
            merkle_tree_hash: ContentHash::default(),
            vector_clock_seen: None,
            vector_clock_write: VectorClock::new(change_set)?,
        })
    }

    pub fn new_with_seen_vector_clock(
        change_set: &ChangeSet,
        kind: ContentKind,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id: change_set.generate_ulid()?,
            origin_id: change_set.generate_ulid()?,
            kind,
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
            ContentKind::Component(_) => ContentKind::Component(content_hash),
            ContentKind::Func(_) => ContentKind::Func(content_hash),
            ContentKind::FuncArg(_) => ContentKind::FuncArg(content_hash),
            ContentKind::Prop(_) => ContentKind::Prop(content_hash),
            ContentKind::Root => return Err(NodeWeightError::CannotUpdateRootNodeContentHash),
            ContentKind::Schema(_) => ContentKind::Schema(content_hash),
            ContentKind::SchemaVariant(_) => ContentKind::SchemaVariant(content_hash),
        };

        self.kind = new_kind;

        Ok(())
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn kind(&self) -> ContentKind {
        self.kind
    }

    pub fn merkle_tree_hash(&self) -> ContentHash {
        self.merkle_tree_hash
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn increment_vector_clocks(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        let new_vc_entry = change_set.generate_ulid()?;

        self.vector_clock_write.inc_to(change_set, new_vc_entry);
        if let Some(vcs) = &mut self.vector_clock_seen {
            vcs.inc_to(change_set, new_vc_entry);
        };

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

    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSet,
        other: &ContentNodeWeight,
    ) -> NodeWeightResult<()> {
        self.vector_clock_write
            .merge(change_set, &other.vector_clock_write)?;

        if let Some(local_vector_clock_seen) = &mut self.vector_clock_seen {
            if let Some(remote_vector_clock_seen) = &other.vector_clock_seen {
                local_vector_clock_seen.merge(change_set, remote_vector_clock_seen)?;
            } else {
                return Err(NodeWeightError::NoSeenVectorClock);
            }
        }

        Ok(())
    }

    pub fn vector_clock_seen(&self) -> Option<&VectorClock> {
        self.vector_clock_seen.as_ref()
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        &self.vector_clock_write
    }
}

impl std::fmt::Debug for ContentNodeWeight {
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

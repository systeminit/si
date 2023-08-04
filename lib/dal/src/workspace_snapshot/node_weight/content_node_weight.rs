use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::workspace_snapshot::{
    change_set::ChangeSet,
    node_weight::{NodeWeightError, NodeWeightResult},
    vector_clock::VectorClock,
};
use crate::ContentHash;

pub type LineageId = Ulid;

#[remain::sorted]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
/// The type of the object, and the content-addressable-storage address (content hash)
/// of the object itself.
pub enum ContentAddress {
    Component(ContentHash),
    Func(ContentHash),
    FuncArg(ContentHash),
    Prop(ContentHash),
    Root,
    Schema(ContentHash),
    SchemaVariant(ContentHash),
}

impl ContentAddress {
    fn content_hash(&self) -> ContentHash {
        match self {
            ContentAddress::Component(id) => Some(*id),
            ContentAddress::Func(id) => Some(*id),
            ContentAddress::FuncArg(id) => Some(*id),
            ContentAddress::Prop(id) => Some(*id),
            ContentAddress::Root => None,
            ContentAddress::Schema(id) => Some(*id),
            ContentAddress::SchemaVariant(id) => Some(*id),
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
    lineage_id: LineageId,
    /// What type of thing is this node representing, and what is the content hash used to
    /// retrieve the data for this specific node.
    content_address: ContentAddress,
    /// [Merkle tree](https://en.wikipedia.org/wiki/Merkle_tree) hash for the graph
    /// starting with this node as the root. Mainly useful in quickly determining "has
    /// something changed anywhere in this (sub)graph".
    merkle_tree_hash: ContentHash,
    /// The first time a [`ChangeSet`] has "seen" this content. This is useful for determining
    /// whether the absence of this content on one side or the other of a rebase/merge is because
    /// the content is new, or because one side deleted it.
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl ContentNodeWeight {
    pub fn new(
        change_set: &ChangeSet,
        id: Ulid,
        content_address: ContentAddress,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id: change_set.generate_ulid()?,
            content_address,
            merkle_tree_hash: ContentHash::default(),
            vector_clock_first_seen: VectorClock::new(change_set)?,
            vector_clock_recently_seen: VectorClock::new(change_set)?,
            vector_clock_write: VectorClock::new(change_set)?,
        })
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn increment_vector_clock(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        self.vector_clock_write.inc(change_set)?;
        self.vector_clock_recently_seen.inc(change_set)?;

        Ok(())
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn mark_seen(&mut self, change_set: &ChangeSet) -> NodeWeightResult<()> {
        self.vector_clock_recently_seen
            .inc(change_set)
            .map_err(Into::into)
    }

    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSet,
        other: &ContentNodeWeight,
    ) -> NodeWeightResult<()> {
        self.vector_clock_write
            .merge(change_set, &other.vector_clock_write)?;
        self.vector_clock_first_seen
            .merge(change_set, &other.vector_clock_first_seen)?;
        self.vector_clock_recently_seen
            .merge(change_set, &other.vector_clock_recently_seen)?;

        Ok(())
    }

    pub fn merkle_tree_hash(&self) -> ContentHash {
        self.merkle_tree_hash
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_kind = match &self.content_address {
            ContentAddress::Component(_) => ContentAddress::Component(content_hash),
            ContentAddress::Func(_) => ContentAddress::Func(content_hash),
            ContentAddress::FuncArg(_) => ContentAddress::FuncArg(content_hash),
            ContentAddress::Prop(_) => ContentAddress::Prop(content_hash),
            ContentAddress::Root => return Err(NodeWeightError::CannotUpdateRootNodeContentHash),
            ContentAddress::Schema(_) => ContentAddress::Schema(content_hash),
            ContentAddress::SchemaVariant(_) => ContentAddress::SchemaVariant(content_hash),
        };

        self.content_address = new_kind;

        Ok(())
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSet,
    ) -> NodeWeightResult<Self> {
        let mut new_node_weight = self.clone();
        new_node_weight.increment_vector_clock(change_set)?;

        Ok(new_node_weight)
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn set_vector_clock_recently_seen_to(
        &mut self,
        change_set: &ChangeSet,
        new_val: DateTime<Utc>,
    ) {
        self.vector_clock_recently_seen.inc_to(change_set, new_val);
    }

    pub fn vector_clock_first_seen(&self) -> &VectorClock {
        &self.vector_clock_first_seen
    }

    pub fn vector_clock_recently_seen(&self) -> &VectorClock {
        &self.vector_clock_recently_seen
    }

    pub fn vector_clock_write(&self) -> &VectorClock {
        &self.vector_clock_write
    }
}

impl std::fmt::Debug for ContentNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("NodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("content_address", &self.content_address)
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .field("vector_clock_first_seen", &self.vector_clock_first_seen)
            .field(
                "vector_clock_recently_seen",
                &self.vector_clock_recently_seen,
            )
            .field("vector_clock_write", &self.vector_clock_write)
            .finish()
    }
}

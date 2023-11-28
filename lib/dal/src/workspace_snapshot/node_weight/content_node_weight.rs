use chrono::{DateTime, Utc};
use content_store::ContentHash;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::{
    change_set_pointer::ChangeSetPointer,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::{NodeWeightError, NodeWeightResult},
        vector_clock::VectorClock,
    },
};

#[derive(Clone, Serialize, Deserialize)]
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
    /// The first time a [`ChangeSetPointer`] has "seen" this content. This is useful for determining
    /// whether the absence of this content on one side or the other of a rebase/merge is because
    /// the content is new, or because one side deleted it.
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl ContentNodeWeight {
    pub fn new(
        change_set: &ChangeSetPointer,
        id: Ulid,
        content_address: ContentAddress,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id: change_set.generate_ulid()?,
            content_address,
            merkle_tree_hash: ContentHash::default(),
            vector_clock_first_seen: VectorClock::new(change_set.vector_clock_id())?,
            vector_clock_recently_seen: VectorClock::new(change_set.vector_clock_id())?,
            vector_clock_write: VectorClock::new(change_set.vector_clock_id())?,
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

    pub fn increment_vector_clock(
        &mut self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<()> {
        self.vector_clock_write.inc(change_set.vector_clock_id())?;
        self.vector_clock_recently_seen
            .inc(change_set.vector_clock_id())?;

        Ok(())
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn mark_seen_at(&mut self, vector_clock_id: VectorClockId, seen_at: DateTime<Utc>) {
        self.vector_clock_recently_seen
            .inc_to(vector_clock_id, seen_at);
        if self
            .vector_clock_first_seen
            .entry_for(vector_clock_id)
            .is_none()
        {
            self.vector_clock_first_seen
                .inc_to(vector_clock_id, seen_at);
        }
    }

    pub fn merge_clocks(
        &mut self,
        change_set: &ChangeSetPointer,
        other: &Self,
    ) -> NodeWeightResult<()> {
        self.vector_clock_write
            .merge(change_set.vector_clock_id(), &other.vector_clock_write)?;
        self.vector_clock_first_seen
            .merge(change_set.vector_clock_id(), &other.vector_clock_first_seen)?;
        self.vector_clock_recently_seen.merge(
            change_set.vector_clock_id(),
            &other.vector_clock_recently_seen,
        )?;

        Ok(())
    }

    pub fn merkle_tree_hash(&self) -> ContentHash {
        self.merkle_tree_hash
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::ActionPrototype(_) => ContentAddress::ActionPrototype(content_hash),
            ContentAddress::AttributePrototype(_) => {
                ContentAddress::AttributePrototype(content_hash)
            }
            ContentAddress::AttributePrototypeArgument(_) => {
                ContentAddress::AttributePrototypeArgument(content_hash)
            }
            ContentAddress::AttributeValue(_) => ContentAddress::AttributeValue(content_hash),
            ContentAddress::Component(_) => ContentAddress::Component(content_hash),
            ContentAddress::ExternalProvider(_) => ContentAddress::ExternalProvider(content_hash),
            ContentAddress::FuncArg(_) => ContentAddress::FuncArg(content_hash),
            ContentAddress::Func(_) => ContentAddress::Func(content_hash),
            ContentAddress::InternalProvider(_) => ContentAddress::InternalProvider(content_hash),
            ContentAddress::Node(_) => ContentAddress::Node(content_hash),
            ContentAddress::Prop(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Prop".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::Root => return Err(NodeWeightError::CannotUpdateRootNodeContentHash),
            ContentAddress::Schema(_) => ContentAddress::Schema(content_hash),
            ContentAddress::SchemaVariant(_) => ContentAddress::SchemaVariant(content_hash),
            ContentAddress::StaticArgumentValue(_) => {
                ContentAddress::StaticArgumentValue(content_hash)
            }
            ContentAddress::Socket(_) => ContentAddress::Socket(content_hash),
            ContentAddress::ValidationPrototype(_) => {
                ContentAddress::ValidationPrototype(content_hash)
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn new_with_incremented_vector_clock(
        &self,
        change_set: &ChangeSetPointer,
    ) -> NodeWeightResult<Self> {
        let mut new_node_weight = self.clone();
        new_node_weight.increment_vector_clock(change_set)?;

        Ok(new_node_weight)
    }

    pub fn node_hash(&self) -> ContentHash {
        self.content_hash()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: ContentHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn set_vector_clock_recently_seen_to(
        &mut self,
        change_set: &ChangeSetPointer,
        new_val: DateTime<Utc>,
    ) {
        self.vector_clock_recently_seen
            .inc_to(change_set.vector_clock_id(), new_val);
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
        f.debug_struct("ContentNodeWeight")
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

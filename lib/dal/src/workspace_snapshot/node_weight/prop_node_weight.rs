use serde::{Deserialize, Serialize};
use si_events::VectorClockId;
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::vector_clock::HasVectorClocks;
use crate::EdgeWeightKindDiscriminants;
use crate::{
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::{NodeWeightError, NodeWeightResult},
        vector_clock::VectorClock,
    },
    PropKind,
};

use super::deprecated::DeprecatedPropNodeWeight;
use super::traits::UpdateConflictsAndUpdates;

#[derive(Clone, Serialize, Deserialize)]
pub struct PropNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    kind: PropKind,
    name: String,
    can_be_used_as_prototype_arg: bool,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl PropNodeWeight {
    pub fn new(
        vector_clock_id: VectorClockId,
        id: Ulid,
        lineage_id: Ulid,
        content_address: ContentAddress,
        kind: PropKind,
        name: String,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            kind,
            name,
            can_be_used_as_prototype_arg: false,
            vector_clock_first_seen: VectorClock::new(vector_clock_id),
            vector_clock_recently_seen: VectorClock::new(vector_clock_id),
            vector_clock_write: VectorClock::new(vector_clock_id),
        })
    }

    pub fn kind(&self) -> PropKind {
        self.kind
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![self.content_address.content_hash()]
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn can_be_used_as_prototype_arg(&self) -> bool {
        self.can_be_used_as_prototype_arg
    }

    pub fn set_can_be_used_as_prototype_arg(&mut self, can_be_used: bool) {
        self.can_be_used_as_prototype_arg = can_be_used;
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::Prop(_) => ContentAddress::Prop(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::Prop.to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "kind": self.kind,
            "name": self.name,
        }])
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl HasVectorClocks for PropNodeWeight {
    fn vector_clock_first_seen(&self) -> &VectorClock {
        &self.vector_clock_first_seen
    }

    fn vector_clock_recently_seen(&self) -> &VectorClock {
        &self.vector_clock_recently_seen
    }

    fn vector_clock_write(&self) -> &VectorClock {
        &self.vector_clock_write
    }

    fn vector_clock_first_seen_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock_first_seen
    }

    fn vector_clock_recently_seen_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock_recently_seen
    }

    fn vector_clock_write_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock_write
    }
}

impl std::fmt::Debug for PropNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PropNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("kind", &self.kind)
            .field("name", &self.name)
            .field("content_hash", &self.content_hash())
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

impl From<DeprecatedPropNodeWeight> for PropNodeWeight {
    fn from(value: DeprecatedPropNodeWeight) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            kind: value.kind,
            name: value.name,
            can_be_used_as_prototype_arg: value.can_be_used_as_prototype_arg,
            vector_clock_first_seen: VectorClock::empty(),
            vector_clock_recently_seen: VectorClock::empty(),
            vector_clock_write: VectorClock::empty(),
        }
    }
}

impl UpdateConflictsAndUpdates for PropNodeWeight {}

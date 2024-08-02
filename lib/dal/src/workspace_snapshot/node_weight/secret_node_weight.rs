use serde::{Deserialize, Serialize};
use si_events::VectorClockId;
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash, EncryptedSecretKey};

use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::vector_clock::HasVectorClocks;
use crate::workspace_snapshot::{
    content_address::ContentAddress,
    graph::LineageId,
    node_weight::{NodeWeightError, NodeWeightResult},
    vector_clock::VectorClock,
};
use crate::EdgeWeightKindDiscriminants;

use super::deprecated::DeprecatedSecretNodeWeight;
use super::traits::UpdateConflictsAndUpdates;

#[derive(Clone, Serialize, Deserialize)]
pub struct SecretNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
    encrypted_secret_key: EncryptedSecretKey,
}

impl SecretNodeWeight {
    pub fn new(
        vector_clock_id: VectorClockId,
        id: Ulid,
        lineage_id: Ulid,
        content_address: ContentAddress,
        encrypted_secret_key: EncryptedSecretKey,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            vector_clock_first_seen: VectorClock::new(vector_clock_id),
            vector_clock_recently_seen: VectorClock::new(vector_clock_id),
            vector_clock_write: VectorClock::new(vector_clock_id),
            encrypted_secret_key,
        })
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![self.content_address.content_hash()]
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

    pub fn encrypted_secret_key(&self) -> EncryptedSecretKey {
        self.encrypted_secret_key
    }

    pub fn set_encrypted_secret_key(
        &mut self,
        encrypted_secret_key: EncryptedSecretKey,
    ) -> &mut Self {
        self.encrypted_secret_key = encrypted_secret_key;
        self
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::Secret(_) => ContentAddress::Secret(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::Secret.to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "encrypted_secret_key": self.encrypted_secret_key,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Use]
    }
}

impl HasVectorClocks for SecretNodeWeight {
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

impl std::fmt::Debug for SecretNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("SecretNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("content_hash", &self.content_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .field("vector_clock_first_seen", &self.vector_clock_first_seen)
            .field(
                "vector_clock_recently_seen",
                &self.vector_clock_recently_seen,
            )
            .field("vector_clock_write", &self.vector_clock_write)
            .field("encrypted_secret_key", &self.encrypted_secret_key)
            .finish()
    }
}

impl From<DeprecatedSecretNodeWeight> for SecretNodeWeight {
    fn from(value: DeprecatedSecretNodeWeight) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            vector_clock_first_seen: VectorClock::empty(),
            vector_clock_recently_seen: VectorClock::empty(),
            vector_clock_write: VectorClock::empty(),
            encrypted_secret_key: value.encrypted_secret_key,
        }
    }
}

impl UpdateConflictsAndUpdates for SecretNodeWeight {}

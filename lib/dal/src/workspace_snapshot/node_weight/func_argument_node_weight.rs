use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    workspace_snapshot::{
        content_address::{ContentAddress, ContentAddressDiscriminants},
        graph::{deprecated::v1::DeprecatedFuncArgumentNodeWeightV1, LineageId},
        node_weight::traits::CorrectTransforms,
        node_weight::NodeWeightResult,
        NodeWeightError,
    },
    EdgeWeightKindDiscriminants,
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FuncArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    name: String,
}

impl FuncArgumentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_address: ContentAddress, name: String) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            name,
        }
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::FuncArg(_) => ContentAddress::FuncArg(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::FuncArg.to_string(),
                )
                .into());
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(self.content_address.content_hash().as_bytes());
        content_hasher.update(self.name.as_bytes());

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl std::fmt::Debug for FuncArgumentNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FuncNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("name", &self.name)
            .field("content_hash", &self.content_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl From<DeprecatedFuncArgumentNodeWeightV1> for FuncArgumentNodeWeight {
    fn from(value: DeprecatedFuncArgumentNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            name: value.name,
        }
    }
}

impl CorrectTransforms for FuncArgumentNodeWeight {}

use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    layer_db_types::FuncArgumentContent, workspace_snapshot::{
        content_address::ContentAddress,
        graph::{deprecated::v1::DeprecatedFuncArgumentNodeWeightV1, LineageId},
        node_weight::{impl_has_content, traits::CorrectTransforms, NodeHash},
    }, EdgeWeightKindDiscriminants
};

use super::HasContentHash as _;


#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FuncArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    pub(super) merkle_tree_hash: MerkleTreeHash,
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

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl NodeHash for FuncArgumentNodeWeight {
    fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "name": self.name,
        }])
    }
}

impl_has_content! { FuncArgumentNodeWeight => FuncArgumentContent }

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

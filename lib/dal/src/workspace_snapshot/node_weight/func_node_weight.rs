use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{func::FuncKind, workspace_snapshot::{content_address::ContentAddress, graph::{deprecated::v1::DeprecatedFuncNodeWeightV1, LineageId}, node_weight::{impl_has_discriminated_content_address, traits::CorrectTransforms, NodeHash}}, EdgeWeightKindDiscriminants};

use super::HasContent as _;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FuncNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    pub(super) merkle_tree_hash: MerkleTreeHash,
    name: String,
    func_kind: FuncKind,
}

impl FuncNodeWeight {
    pub fn new(
        id: Ulid,
        lineage_id: Ulid,
        content_address: ContentAddress,
        name: String,
        func_kind: FuncKind,
    ) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            name,
            func_kind,
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

    pub fn func_kind(&self) -> FuncKind {
        self.func_kind
    }

    pub fn set_func_kind(&mut self, func_kind: FuncKind) -> &mut Self {
        self.func_kind = func_kind;
        self
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl NodeHash for FuncNodeWeight {
    fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "name": self.name,
            "func_kind": self.func_kind,
        }])
    }
}

impl_has_discriminated_content_address! { FuncNodeWeight: Func }

impl std::fmt::Debug for FuncNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FuncNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("name", &self.name)
            .field("func_kind", &self.func_kind)
            .field("content_hash", &self.content_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl From<DeprecatedFuncNodeWeightV1> for FuncNodeWeight {
    fn from(value: DeprecatedFuncNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            name: value.name,
            func_kind: value.func_kind,
        }
    }
}

impl CorrectTransforms for FuncNodeWeight {}

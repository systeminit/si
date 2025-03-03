use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::graph::deprecated::v1::DeprecatedPropNodeWeightV1;
use crate::EdgeWeightKindDiscriminants;
use crate::{
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::traits::CorrectTransforms,
        node_weight::{NodeWeightError, NodeWeightResult},
    },
    PropKind,
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    kind: PropKind,
    name: String,
    can_be_used_as_prototype_arg: bool,
}

impl PropNodeWeight {
    pub fn new(
        id: Ulid,
        lineage_id: Ulid,
        content_address: ContentAddress,
        kind: PropKind,
        name: String,
    ) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            kind,
            name,
            can_be_used_as_prototype_arg: false,
        }
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
        content_hasher.update(self.kind.to_string().as_bytes());
        content_hasher.update(self.name.as_bytes());

        content_hasher.finalize()
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
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
            .finish()
    }
}

impl From<DeprecatedPropNodeWeightV1> for PropNodeWeight {
    fn from(value: DeprecatedPropNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            kind: value.kind,
            name: value.name,
            can_be_used_as_prototype_arg: value.can_be_used_as_prototype_arg,
        }
    }
}

impl CorrectTransforms for PropNodeWeight {}

use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::{
    EdgeWeightKindDiscriminants,
    workspace_snapshot::{
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        graph::LineageId,
        node_weight::{
            NodeWeightError,
            NodeWeightResult,
            traits::CorrectTransforms,
        },
    },
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContentNodeWeight {
    /// The stable local ID of the object in question. Mainly used by external things like
    /// the UI to be able to say "do X to _this_ thing" since the `NodeIndex` is an
    /// internal implementation detail, and the content ID wrapped by the
    /// [`NodeWeightKind`] changes whenever something about the node itself changes (for
    /// example, the name, or type of a [`Prop`].)
    pub id: Ulid,
    /// Globally stable ID for tracking the "lineage" of a thing to determine whether it
    /// should be trying to receive updates.
    pub lineage_id: LineageId,
    /// What type of thing is this node representing, and what is the content hash used to
    /// retrieve the data for this specific node.
    pub content_address: ContentAddress,
    /// [Merkle tree](https://en.wikipedia.org/wiki/Merkle_tree) hash for the graph
    /// starting with this node as the root. Mainly useful in quickly determining "has
    /// something changed anywhere in this (sub)graph".
    pub merkle_tree_hash: MerkleTreeHash,
    /// The first time a [`ChangeSet`] has "seen" this content. This is useful for determining
    /// whether the absence of this content on one side or the other of a rebase/merge is because
    /// the content is new, or because one side deleted it.
    pub to_delete: bool,
}

impl ContentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_address: ContentAddress) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            to_delete: false,
        }
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_address_discriminants(&self) -> ContentAddressDiscriminants {
        self.content_address.into()
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
    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::DeprecatedAction(_) => ContentAddress::DeprecatedAction(content_hash),
            ContentAddress::DeprecatedActionBatch(_) => {
                ContentAddress::DeprecatedActionBatch(content_hash)
            }
            ContentAddress::DeprecatedActionRunner(_) => {
                ContentAddress::DeprecatedActionRunner(content_hash)
            }
            ContentAddress::ActionPrototype(_) => ContentAddress::ActionPrototype(content_hash),
            ContentAddress::ApprovalRequirementDefinition(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "ApprovalRequirement".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::AttributePrototype(_) => {
                ContentAddress::AttributePrototype(content_hash)
            }
            ContentAddress::Component(_) => ContentAddress::Component(content_hash),
            ContentAddress::OutputSocket(_) => ContentAddress::OutputSocket(content_hash),
            ContentAddress::FuncArg(_) => ContentAddress::FuncArg(content_hash),
            ContentAddress::Func(_) => ContentAddress::Func(content_hash),
            ContentAddress::Geometry(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Geometry".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::InputSocket(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "InputSocket".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::JsonValue(_) => ContentAddress::JsonValue(content_hash),
            ContentAddress::Module(_) => ContentAddress::Module(content_hash),
            ContentAddress::Prop(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Prop".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::Root => return Err(NodeWeightError::CannotUpdateRootNodeContentHash),
            ContentAddress::Schema(_) => ContentAddress::Schema(content_hash),
            ContentAddress::SchemaVariant(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "SchemaVariant".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::Secret(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Secret".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::StaticArgumentValue(_) => {
                ContentAddress::StaticArgumentValue(content_hash)
            }
            ContentAddress::ValidationPrototype(_) => {
                ContentAddress::ValidationPrototype(content_hash)
            }
            ContentAddress::ValidationOutput(_) => ContentAddress::ValidationOutput(content_hash),
            ContentAddress::ManagementPrototype(_) => {
                ContentAddress::ManagementPrototype(content_hash)
            }
            ContentAddress::View(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Geometry".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::AttributePaths(_) => ContentAddress::AttributePaths(content_hash),
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        self.content_hash()
    }
    pub fn set_to_delete(&mut self, to_delete: bool) -> bool {
        self.to_delete = to_delete;
        self.to_delete
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl std::fmt::Debug for ContentNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ContentNodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("content_address", &self.content_address)
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl CorrectTransforms for ContentNodeWeight {}

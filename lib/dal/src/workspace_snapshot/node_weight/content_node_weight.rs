use serde::{Deserialize, Serialize};
use si_events::merkle_tree_hash::MerkleTreeHash;
use si_events::{ulid::Ulid, ContentHash};

use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::graph::deprecated::v1::DeprecatedContentNodeWeightV1;
use crate::workspace_snapshot::{
    content_address::ContentAddress,
    graph::LineageId,
    node_weight::traits::CorrectTransforms,
    node_weight::{NodeWeightError, NodeWeightResult},
};
use crate::EdgeWeightKindDiscriminants;

use super::{HasContent, HasContentAddress, NodeHash};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    content_address: ContentAddress,
    /// [Merkle tree](https://en.wikipedia.org/wiki/Merkle_tree) hash for the graph
    /// starting with this node as the root. Mainly useful in quickly determining "has
    /// something changed anywhere in this (sub)graph".
    pub(super) merkle_tree_hash: MerkleTreeHash,
    /// The first time a [`ChangeSet`] has "seen" this content. This is useful for determining
    /// whether the absence of this content on one side or the other of a rebase/merge is because
    /// the content is new, or because one side deleted it.
    to_delete: bool,
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

    pub fn id(&self) -> Ulid {
        self.id
    }
    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub fn set_to_delete(&mut self, to_delete: bool) -> bool {
        self.to_delete = to_delete;
        self.to_delete
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl NodeHash for ContentNodeWeight {
    fn node_hash(&self) -> ContentHash { self.content_hash() }
}

impl HasContent for ContentNodeWeight {
    fn content_hash(&self) -> ContentHash { self.content_address.content_hash() }
    fn content_store_hashes(&self) -> Vec<ContentHash> { vec![self.content_address.content_hash()] }
}

impl HasContentAddress for ContentNodeWeight {
    fn content_address(&self) -> ContentAddress { self.content_address }
    fn content_address_discriminants(&self) -> ContentAddressDiscriminants { self.content_address.into() }
    fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::DeprecatedAction(_) => ContentAddress::DeprecatedAction(content_hash),
            ContentAddress::DeprecatedActionBatch(_) => {
                ContentAddress::DeprecatedActionBatch(content_hash)
            }
            ContentAddress::DeprecatedActionRunner(_) => {
                ContentAddress::DeprecatedActionRunner(content_hash)
            }
            ContentAddress::ActionPrototype(_) => ContentAddress::ActionPrototype(content_hash),
            ContentAddress::AttributePrototype(_) => {
                ContentAddress::AttributePrototype(content_hash)
            }
            ContentAddress::Component(_) => ContentAddress::Component(content_hash),
            ContentAddress::OutputSocket(_) => ContentAddress::OutputSocket(content_hash),
            ContentAddress::FuncArg(_) => ContentAddress::FuncArg(content_hash),
            ContentAddress::Func(_) => ContentAddress::Func(content_hash),
            ContentAddress::InputSocket(_) => ContentAddress::InputSocket(content_hash),
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
            ContentAddress::SchemaVariant(_) => ContentAddress::SchemaVariant(content_hash),
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
        };

        self.content_address = new_address;

        Ok(())
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

impl From<DeprecatedContentNodeWeightV1> for ContentNodeWeight {
    fn from(value: DeprecatedContentNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            to_delete: value.to_delete,
        }
    }
}

impl CorrectTransforms for ContentNodeWeight {}

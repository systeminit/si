use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};
use strum::{
    Display,
    EnumIter,
};

use crate::{
    EdgeWeightKindDiscriminants,
    workspace_snapshot::{
        graph::LineageId,
        node_weight::traits::CorrectTransforms,
    },
};

/// NOTE: adding new categories can be done in a backwards compatible way, so
/// long as we don't assume the new categories already exists on the graph. In
/// places where you need to access the category, check if it exists, and if it
/// doesn't exist, create it (if it makes sense to do so in the given context).
/// By giving the new categories a static Ulid, (see below), you can avoid any
/// conflicts with duplicate category nodes, since the static ids for the
/// category nodes will have all of their outgoing edges merged on rebase. When
/// generating a static ulid, use a timestamp far in the past, so that there is
/// no potential of conflict with an existing Ulid in the system.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumIter, Hash)]
pub enum CategoryNodeKind {
    Action,
    Component,
    DeprecatedActionBatch,
    Func,
    Module,
    Schema,
    Secret,
    DependentValueRoots,
    View,
    DiagramObject,
    DefaultSubscriptionSources,
    Overlays,
}

const DEFAULT_SUBSCRIPTION_SOURCE_CATEGORY_ID_STR: &str = "0000DNP8N22X0A8S4CV2APWX3A";
const OVERLAY_CATEGORY_ID_STR: &str = "0000BQJPTG3PZ3XQG9M3W7BSDR";

impl CategoryNodeKind {
    /// Adding a new category node to an existing workspacewithout migrating a
    /// workspace and all its change set requires that the category node have a
    /// static id, so that the rebaser will treat all versions of it in every
    /// change set as the same node.
    pub fn static_id(&self) -> Option<Ulid> {
        match self {
            CategoryNodeKind::Action
            | CategoryNodeKind::Component
            | CategoryNodeKind::DeprecatedActionBatch
            | CategoryNodeKind::Func
            | CategoryNodeKind::Module
            | CategoryNodeKind::Schema
            | CategoryNodeKind::Secret
            | CategoryNodeKind::DependentValueRoots
            | CategoryNodeKind::View
            | CategoryNodeKind::DiagramObject => None,
            CategoryNodeKind::Overlays => Ulid::from_string(OVERLAY_CATEGORY_ID_STR).ok(),
            CategoryNodeKind::DefaultSubscriptionSources => {
                Ulid::from_string(DEFAULT_SUBSCRIPTION_SOURCE_CATEGORY_ID_STR).ok()
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CategoryNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    kind: CategoryNodeKind,
    merkle_tree_hash: MerkleTreeHash,
}

impl CategoryNodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn kind(&self) -> CategoryNodeKind {
        self.kind
    }

    pub fn set_kind(&mut self, kind: CategoryNodeKind) {
        self.kind = kind;
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

    pub fn new(id: Ulid, lineage_id: Ulid, kind: CategoryNodeKind) -> Self {
        Self {
            id,
            lineage_id,
            kind,
            merkle_tree_hash: Default::default(),
        }
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(self.kind.to_string().as_bytes());

        content_hasher.finalize()
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl std::fmt::Debug for CategoryNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("CategoryNodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl CorrectTransforms for CategoryNodeWeight {}

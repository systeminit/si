use std::{
    fs::File,
    io::Write,
};

use detector::Update;
use petgraph::prelude::*;
/// Ensure [`NodeIndex`], and [`Direction`] are usable externally.
pub use petgraph::{
    Direction,
    graph::NodeIndex,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};
use si_id::{
    ApprovalRequirementDefinitionId,
    EntityId,
};
use si_layer_cache::{
    LayerDbError,
    db::serialize,
};
use strum::{
    EnumDiscriminants,
    EnumIter,
    EnumString,
    IntoEnumIterator,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    ComponentError,
    EdgeWeightKindDiscriminants,
    SchemaVariantError,
    socket::input::InputSocketError,
    workspace_snapshot::node_weight::{
        NodeWeightError,
        category_node_weight::CategoryNodeKind,
    },
};

pub mod correct_transforms;
pub mod detector;
mod tests;
pub mod traits;
pub mod v3;
pub mod v4;
pub mod validator;

pub use traits::{
    approval_requirement::ApprovalRequirementExt,
    schema::variant::SchemaVariantExt,
    socket::input::InputSocketExt,
};
pub use v3::WorkspaceSnapshotGraphV3;
pub use v4::WorkspaceSnapshotGraphV4;

pub type LineageId = Ulid;
pub type WorkspaceSnapshotGraphVCurrent = WorkspaceSnapshotGraphV4;

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceSnapshotGraphError {
    #[error(
        "Cannot compare ordering of container elements between ordered, and un-ordered container: {0:?}, {1:?}"
    )]
    CannotCompareOrderedAndUnorderedContainers(NodeIndex, NodeIndex),
    #[error("could not find category node of kind: {0:?}")]
    CategoryNodeNotFound(CategoryNodeKind),
    #[error("Component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("Unable to retrieve content for ContentHash")]
    ContentMissingForContentHash,
    #[error("Action would create a graph cycle")]
    CreateGraphCycle,
    #[error("could not find the newly imported subgraph when performing updates")]
    DestinationNotUpdatedWhenImportingSubgraph,
    #[error("Edge does not exist for EdgeIndex: {0:?}")]
    EdgeDoesNotExist(EdgeIndex),
    #[error("EdgeWeight not found")]
    EdgeWeightNotFound,
    #[error("Entity not found for Approval Requirement Definition Id: {0}")]
    EntityNotFoundForApprovalRequirementDefinition(ApprovalRequirementDefinitionId),
    #[error("Problem during graph traversal: {0:?}")]
    GraphTraversal(petgraph::visit::DfsEvent<NodeIndex>),
    #[error("Incompatible node types")]
    IncompatibleNodeTypes,
    #[error("InputSocket error {0}")]
    InputSocketError(#[from] Box<InputSocketError>),
    #[error("Invalid value graph")]
    InvalidValueGraph,
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error(
        "multiple merkle tree hashes found for entity {0} (at least two found, including {1} and {2})"
    )]
    MultipleMerkleTreeHashesForEntity(EntityId, MerkleTreeHash, MerkleTreeHash),
    #[error("mutex poisoning: {0}")]
    MutexPoison(String),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("node weight not found")]
    NodeWeightNotFound,
    #[error("Node with ID {} not found", .0.to_string())]
    NodeWithIdNotFound(Ulid),
    #[error("No edges of kind {1} found with node index {0:?} as the source")]
    NoEdgesOfKindFound(NodeIndex, EdgeWeightKindDiscriminants),
    #[error("No Prop found for NodeIndex {0:?}")]
    NoPropFound(NodeIndex),
    #[error("Ordering node {0} has id in its order for non-existent node {1}")]
    OrderingNodeHasNonexistentNodeInOrder(Ulid, Ulid),
    #[error("SchemaVariant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("Too many edges of kind {1} found with node index {0:?} as the source")]
    TooManyEdgesOfKind(NodeIndex, EdgeWeightKindDiscriminants),
    #[error("NodeIndex has too many Ordering children: {0:?}")]
    TooManyOrderingForNode(NodeIndex),
    #[error("NodeIndex has too many Prop children: {0:?}")]
    TooManyPropForNode(NodeIndex),
    #[error("Removing View would orphan items: {0:?}")]
    ViewRemovalWouldOrphanItems(Vec<Ulid>),
    #[error("Workspace Snapshot has conflicts and must be rebased")]
    WorkspaceNeedsRebase,
    #[error("Workspace Snapshot has conflicts")]
    WorkspacesConflict,
}

pub type WorkspaceSnapshotGraphResult<T> = Result<T, WorkspaceSnapshotGraphError>;

#[derive(Debug, Deserialize, Serialize, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize, EnumString, EnumIter))]
pub enum WorkspaceSnapshotGraph {
    Legacy,
    V1,
    V2,
    /// Added `InputSocket` and `SchemaVariant` `NodeWeight` variants.
    V3(WorkspaceSnapshotGraphV3),
    /// Added `View`, `Geometry` and `DiagramObject` categories,
    V4(WorkspaceSnapshotGraphV4),
}

impl std::ops::Deref for WorkspaceSnapshotGraph {
    type Target = WorkspaceSnapshotGraphVCurrent;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl std::ops::DerefMut for WorkspaceSnapshotGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl WorkspaceSnapshotGraph {
    /// Return a reference to the most up to date enum variant for the graph type
    pub fn inner(&self) -> &WorkspaceSnapshotGraphVCurrent {
        match self {
            Self::Legacy | Self::V1 | Self::V2 | Self::V3(_) => {
                unimplemented!("Attempted to access an unmigrated snapshot!")
            }
            Self::V4(inner) => inner,
        }
    }

    pub fn inner_mut(&mut self) -> &mut WorkspaceSnapshotGraphVCurrent {
        match self {
            Self::Legacy | Self::V1 | Self::V2 | Self::V3(_) => {
                unimplemented!("Attempted to access an unmigrated snapshot!")
            }
            Self::V4(inner) => inner,
        }
    }

    pub fn current_discriminant() -> WorkspaceSnapshotGraphDiscriminants {
        WorkspaceSnapshotGraphDiscriminants::iter()
            .next_back()
            .expect("Unable to get last element of an iterator guaranteed to have elements")
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RebaseBatch {
    updates: Vec<Update>,
}

impl RebaseBatch {
    pub fn new(updates: Vec<Update>) -> Self {
        Self { updates }
    }

    pub fn updates(&self) -> &[Update] {
        &self.updates
    }

    /// Write the rebase batch to disk. This *MAY PANIC*. Use only for
    /// debugging.
    #[allow(clippy::disallowed_methods)]
    pub fn write_to_disk(&self, file_suffix: &str) {
        let (serialized, _) = serialize::to_vec(self).expect("unable to serialize");
        let filename = format!("{}-{}", Ulid::new(), file_suffix);

        let home_env = std::env::var("HOME").expect("No HOME environment variable set");
        let home = std::path::Path::new(&home_env);
        let mut file = File::create(home.join(&filename)).expect("could not create file");
        file.write_all(&serialized).expect("could not write file");

        println!("Wrote rebase batch to {}", home.join(&filename).display());
    }
}

use si_events::{
    ContentHash,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};
use thiserror::Error;

use super::{
    NodeWeightDiscriminants,
    NodeWeightError,
};
use crate::{
    EdgeWeightKindDiscriminants,
    WorkspaceSnapshotGraphVCurrent,
    workspace_snapshot::{
        NodeInformation,
        graph::{
            WorkspaceSnapshotGraphError,
            detector::Update,
        },
    },
};

pub mod correct_exclusive_outgoing_edge;

pub use correct_exclusive_outgoing_edge::{
    CorrectExclusiveOutgoingEdge,
    ExclusiveOutgoingEdges,
    SplitCorrectExclusiveOutgoingEdge,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CorrectTransformsError {
    #[error("Invalid Updates: {0}")]
    InvalidUpdates(String),
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("expected a node weight of kind {0} but got another, or none")]
    UnexpectedNodeWeight(NodeWeightDiscriminants),
    #[error("workspace snapshot graph: {0}")]
    WorkspaceSnapshotGraph(#[from] WorkspaceSnapshotGraphError),
}

pub type CorrectTransformsResult<T> = Result<T, CorrectTransformsError>;

pub trait CorrectTransforms {
    fn correct_transforms(
        &self,
        _workspace_snapshot_graph: &WorkspaceSnapshotGraphVCurrent,
        updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        Ok(updates)
    }
}

pub trait SiNodeWeight:
    CorrectTransforms + CorrectExclusiveOutgoingEdge + ExclusiveOutgoingEdges + Sized
{
    fn content_hash(&self) -> ContentHash;
    fn id(&self) -> Ulid;
    fn lineage_id(&self) -> Ulid;
    fn merkle_tree_hash(&self) -> MerkleTreeHash;
    fn node_hash(&self) -> ContentHash;
    fn node_weight_discriminant(&self) -> NodeWeightDiscriminants;
    fn set_id(&mut self, new_id: Ulid);
    fn set_lineage_id(&mut self, new_lineage_id: Ulid);
    fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash);

    fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![self.content_hash()]
    }
}

pub trait SiVersionedNodeWeight {
    type Inner: SiNodeWeight;

    /// Return a reference to the most up to date enum variant
    fn inner(&self) -> &Self::Inner;
    /// Return a mutable reference to the most up to date enum variant
    fn inner_mut(&mut self) -> &mut Self::Inner;

    fn id(&self) -> Ulid {
        self.inner().id()
    }

    fn set_id(&mut self, new_id: Ulid) {
        self.inner_mut().set_id(new_id);
    }

    fn lineage_id(&self) -> Ulid {
        self.inner().lineage_id()
    }

    fn set_lineage_id(&mut self, new_lineage_id: Ulid) {
        self.inner_mut().set_lineage_id(new_lineage_id);
    }

    fn node_hash(&self) -> ContentHash {
        self.inner().node_hash()
    }

    fn node_weight_discriminant(&self) -> NodeWeightDiscriminants {
        self.inner().node_weight_discriminant()
    }

    fn content_hash(&self) -> ContentHash {
        self.inner().content_hash()
    }

    fn content_store_hashes(&self) -> Vec<ContentHash> {
        self.inner().content_store_hashes()
    }

    fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.inner().merkle_tree_hash()
    }

    fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.inner_mut().set_merkle_tree_hash(new_hash)
    }
}

impl<T> SiNodeWeight for T
where
    T: SiVersionedNodeWeight,
{
    fn content_hash(&self) -> ContentHash {
        SiVersionedNodeWeight::content_hash(self)
    }

    fn id(&self) -> Ulid {
        SiVersionedNodeWeight::id(self)
    }

    fn lineage_id(&self) -> Ulid {
        SiVersionedNodeWeight::lineage_id(self)
    }

    fn merkle_tree_hash(&self) -> MerkleTreeHash {
        SiVersionedNodeWeight::merkle_tree_hash(self)
    }

    fn node_hash(&self) -> ContentHash {
        SiVersionedNodeWeight::node_hash(self)
    }

    fn node_weight_discriminant(&self) -> NodeWeightDiscriminants {
        SiVersionedNodeWeight::node_weight_discriminant(self)
    }

    fn set_id(&mut self, new_id: Ulid) {
        SiVersionedNodeWeight::set_id(self, new_id)
    }

    fn set_lineage_id(&mut self, new_lineage_id: Ulid) {
        SiVersionedNodeWeight::set_lineage_id(self, new_lineage_id)
    }

    fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        SiVersionedNodeWeight::set_merkle_tree_hash(self, new_hash)
    }

    fn content_store_hashes(&self) -> Vec<ContentHash> {
        SiVersionedNodeWeight::content_store_hashes(self)
    }
}

impl<T> CorrectTransforms for T
where
    T: SiVersionedNodeWeight,
    NodeInformation: for<'a> From<&'a T>,
{
    fn correct_transforms(
        &self,
        workspace_snapshot_graph: &WorkspaceSnapshotGraphVCurrent,
        updates: Vec<Update>,
        from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        self.inner().correct_transforms(
            workspace_snapshot_graph,
            updates,
            from_different_change_set,
        )
    }
}

impl<T> CorrectExclusiveOutgoingEdge for T
where
    T: SiVersionedNodeWeight,
    NodeInformation: for<'a> From<&'a T>,
{
}

impl<T> ExclusiveOutgoingEdges for T
where
    T: SiVersionedNodeWeight,
{
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        self.inner().exclusive_outgoing_edges()
    }
}

impl<T> From<&T> for NodeInformation
where
    T: SiNodeWeight,
{
    fn from(value: &T) -> Self {
        Self {
            node_weight_kind: value.node_weight_discriminant(),
            id: value.id().into(),
        }
    }
}

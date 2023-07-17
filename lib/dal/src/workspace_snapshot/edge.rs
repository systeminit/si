//! Edges

use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type SnapshotEdgeId = Ulid;

#[derive(Debug, Serialize, Deserialize)]
pub enum SnapshotEdgeKind {
    Uses,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotEdge {
    pub id: SnapshotEdgeId,
    pub kind: SnapshotEdgeKind,
}

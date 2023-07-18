//! Edges

use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type SnapshotEdgeId = Ulid;

#[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SnapshotEdgeKind {
    #[default]
    Uses,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SnapshotEdge {
    pub kind: SnapshotEdgeKind,
}

impl SnapshotEdge {
    pub fn new(kind: SnapshotEdgeKind) -> Self {
        Self { kind }
    }
}

//! Nodes

use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type SnapshotNodeId = Ulid;

#[derive(Debug, Serialize, Deserialize)]
pub enum SnapshotNodeKind {
    Root,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotNode {
    pub id: SnapshotNodeId,
    pub kind: SnapshotNodeKind,
}

impl SnapshotNode {
    pub fn new(kind: SnapshotNodeKind) -> SnapshotNode {
        SnapshotNode {
            kind,
            id: SnapshotNodeId::new(),
        }
    }
}

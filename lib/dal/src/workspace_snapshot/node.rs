//! Nodes

use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type SnapshotNodeId = Ulid;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SnapshotNodeKind {
    Root,
    SchemaVariant,
    Schema,
    Component,
    Func,
    Prop,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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

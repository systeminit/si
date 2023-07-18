//! Nodes

use crate::workspace_snapshot::WorkspaceSnapshotResult;
use crate::{ComponentId, FuncId, PropId, SchemaId, SchemaVariantId};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type OriginId = Ulid;

pub type ContentHash = blake3::Hash;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum NodeWeightKind {
    Root,
    SchemaVariant(SchemaVariantId),
    Schema(SchemaId),
    Component(ComponentId),
    Func(FuncId),
    Prop(PropId),
}

impl NodeWeightKind {
    // TODO(nick,jacob): this will only make sense once IDs are the content hashes themselves.
    fn content_hash(&self) -> WorkspaceSnapshotResult<ContentHash> {
        let mut hasher = blake3::Hasher::new();
        let id_as_string = match self {
            NodeWeightKind::Root => None,
            NodeWeightKind::SchemaVariant(id) => Some(id.to_string()),
            NodeWeightKind::Schema(id) => Some(id.to_string()),
            NodeWeightKind::Component(id) => Some(id.to_string()),
            NodeWeightKind::Func(id) => Some(id.to_string()),
            NodeWeightKind::Prop(id) => Some(id.to_string()),
        };
        if let Some(found_id_as_string) = id_as_string {
            hasher.update(found_id_as_string.as_bytes());
        }
        Ok(hasher.finalize())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct NodeWeight {
    pub origin_id: OriginId,
    pub kind: NodeWeightKind,
    // TODO(nick,jacob): this will be handled by the graph.
    // pub merkle_tree_hash: ContentHash,
}

impl NodeWeight {
    pub fn new(kind: NodeWeightKind) -> NodeWeight {
        NodeWeight {
            origin_id: OriginId::new(),
            kind,
        }
    }
}

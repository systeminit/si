use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

use super::WorkspaceSnapshotGraphV4;
use crate::workspace_snapshot::graph::{
    WorkspaceSnapshotGraphResult,
    traits::entity_kind::EntityKindExt,
};

impl EntityKindExt for WorkspaceSnapshotGraphV4 {
    fn get_entity_kind_for_id(&self, id: EntityId) -> WorkspaceSnapshotGraphResult<EntityKind> {
        Ok(self.get_node_weight_by_id(id)?.into())
    }
}

use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

use crate::workspace_snapshot::graph::{
    traits::entity_kind::EntityKindExt, WorkspaceSnapshotGraphResult,
};

use super::WorkspaceSnapshotGraphV4;

impl EntityKindExt for WorkspaceSnapshotGraphV4 {
    fn get_entity_kind_for_id(&self, id: EntityId) -> WorkspaceSnapshotGraphResult<EntityKind> {
        Ok(self.get_node_weight_by_id(id)?.into())
    }
}

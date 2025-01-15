use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphResult;

pub trait EntityKindExt {
    fn get_entity_kind_for_id(&self, id: EntityId) -> WorkspaceSnapshotGraphResult<EntityKind>;
}

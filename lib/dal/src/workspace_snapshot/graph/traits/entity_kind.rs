use si_events::workspace_snapshot::EntityKind;
use si_id::ulid::Ulid;

use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphResult;

pub trait EntityKindExt {
    fn get_entity_kind_for_id(&self, id: Ulid) -> WorkspaceSnapshotGraphResult<EntityKind>;
}

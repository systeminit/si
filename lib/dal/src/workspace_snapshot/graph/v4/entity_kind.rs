use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

use super::WorkspaceSnapshotGraphV4;
use crate::{
    entity_kind::{
        EntityKindError,
        EntityKindResult,
    },
    workspace_snapshot::graph::traits::entity_kind::EntityKindExt,
};

impl EntityKindExt for WorkspaceSnapshotGraphV4 {
    fn get_entity_kind_for_id(&self, id: EntityId) -> EntityKindResult<EntityKind> {
        self.get_node_weight_by_id(id)
            .map_err(|_| EntityKindError::NodeNotFound(id))
            .map(Into::into)
    }
}

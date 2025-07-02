use async_trait::async_trait;
use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

use crate::{
    WorkspaceSnapshot,
    entity_kind::EntityKindResult,
    workspace_snapshot::graph::traits::entity_kind::EntityKindExt as _,
};

#[async_trait]
pub trait EntityKindExt {
    async fn get_entity_kind_for_id(&self, id: EntityId) -> EntityKindResult<EntityKind>;
}

#[async_trait]
impl EntityKindExt for WorkspaceSnapshot {
    async fn get_entity_kind_for_id(&self, id: EntityId) -> EntityKindResult<EntityKind> {
        self.working_copy().await.get_entity_kind_for_id(id)
    }
}

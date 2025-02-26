use anyhow::Result;
use async_trait::async_trait;
use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

use crate::{
    workspace_snapshot::graph::traits::entity_kind::EntityKindExt as EntityKindExtGraph,
    WorkspaceSnapshot,
};

#[async_trait]
pub trait EntityKindExt {
    async fn get_entity_kind_for_id(&self, id: EntityId) -> Result<EntityKind>;
}

#[async_trait]
impl EntityKindExt for WorkspaceSnapshot {
    async fn get_entity_kind_for_id(&self, id: EntityId) -> Result<EntityKind> {
        Ok(self.working_copy().await.get_entity_kind_for_id(id)?)
    }
}

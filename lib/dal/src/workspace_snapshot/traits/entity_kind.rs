use async_trait::async_trait;
use si_events::workspace_snapshot::EntityKind;
use si_id::ulid::Ulid;

use crate::{
    workspace_snapshot::{
        graph::traits::entity_kind::EntityKindExt as EntityKindExtGraph, WorkspaceSnapshotResult,
    },
    WorkspaceSnapshot,
};

#[async_trait]
pub trait EntityKindExt {
    async fn get_entity_kind_for_id(&self, id: Ulid) -> WorkspaceSnapshotResult<EntityKind>;
}

#[async_trait]
impl EntityKindExt for WorkspaceSnapshot {
    async fn get_entity_kind_for_id(&self, id: Ulid) -> WorkspaceSnapshotResult<EntityKind> {
        Ok(self.working_copy().await.get_entity_kind_for_id(id)?)
    }
}

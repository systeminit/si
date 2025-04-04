use async_trait::async_trait;
use si_id::ComponentId;

use crate::{
    diagram::view::ViewId,
    workspace_snapshot::{
        graph::traits::diagram::view::ViewExt as GraphViewExt, WorkspaceSnapshotResult,
    },
    WorkspaceSnapshot,
};

#[async_trait]
pub trait ViewExt {
    async fn view_remove(&self, view_id: ViewId) -> WorkspaceSnapshotResult<()>;

    async fn list_for_component_id(&self, id: ComponentId) -> WorkspaceSnapshotResult<Vec<ViewId>>;
}

#[async_trait]
impl ViewExt for WorkspaceSnapshot {
    async fn view_remove(&self, view_id: ViewId) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut()
            .await
            .view_remove(view_id)
            .map_err(Into::into)
    }

    async fn list_for_component_id(&self, id: ComponentId) -> WorkspaceSnapshotResult<Vec<ViewId>> {
        self.working_copy()
            .await
            .list_for_component_id(id)
            .map_err(Into::into)
    }
}

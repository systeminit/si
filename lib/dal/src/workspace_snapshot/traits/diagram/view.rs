use async_trait::async_trait;

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
}

#[async_trait]
impl ViewExt for WorkspaceSnapshot {
    async fn view_remove(&self, view_id: ViewId) -> WorkspaceSnapshotResult<()> {
        self.working_copy_mut()
            .await
            .view_remove(view_id)
            .map_err(Into::into)
    }
}

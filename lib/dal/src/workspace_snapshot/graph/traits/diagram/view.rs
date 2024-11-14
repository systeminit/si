use crate::{diagram::view::ViewId, workspace_snapshot::graph::WorkspaceSnapshotGraphResult};

pub trait ViewExt {
    fn view_remove(&mut self, view_id: ViewId) -> WorkspaceSnapshotGraphResult<()>;
}

use si_id::ComponentId;

use crate::{
    diagram::view::ViewId,
    workspace_snapshot::graph::WorkspaceSnapshotGraphResult,
};

pub trait ViewExt {
    fn view_remove(&mut self, view_id: ViewId) -> WorkspaceSnapshotGraphResult<()>;

    fn list_for_component_id(
        &self,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotGraphResult<Vec<ViewId>>;
}

use si_id::{AttributeValueId, ComponentId};

use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphResult;

pub trait ComponentExt {
    fn root_attribute_value(
        &self,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueId>;

    fn resolve_attribute_value(
        &self,
        component_id: ComponentId,
        json_pointer: &str,
    ) -> WorkspaceSnapshotGraphResult<Option<AttributeValueId>>;
}

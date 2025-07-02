use async_trait::async_trait;
use si_id::{
    AttributePrototypeId,
    AttributeValueId,
};

use crate::{
    WorkspaceSnapshot,
    workspace_snapshot::{
        WorkspaceSnapshotResult,
        graph::traits::attribute_value::AttributeValueExt as _,
    },
};

#[async_trait]
pub trait AttributeValueExt {
    async fn component_prototype_id(
        &self,
        id: AttributeValueId,
    ) -> WorkspaceSnapshotResult<Option<AttributePrototypeId>>;
}

#[async_trait]
impl AttributeValueExt for WorkspaceSnapshot {
    async fn component_prototype_id(
        &self,
        id: AttributeValueId,
    ) -> WorkspaceSnapshotResult<Option<AttributePrototypeId>> {
        Ok(self.working_copy().await.component_prototype_id(id)?)
    }
}

use async_trait::async_trait;
use si_id::{
    AttributeValueId,
    ComponentId,
};

use crate::{
    WorkspaceSnapshot,
    component::ComponentResult,
    workspace_snapshot::graph::traits::component::ComponentExt as _,
};

#[async_trait]
pub trait ComponentExt {
    async fn root_attribute_value(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<AttributeValueId>;

    async fn external_source_count(&self, id: ComponentId) -> ComponentResult<usize>;
}

#[async_trait]
impl ComponentExt for WorkspaceSnapshot {
    async fn root_attribute_value(
        &self,
        component_id: ComponentId,
    ) -> ComponentResult<AttributeValueId> {
        self.working_copy().await.root_attribute_value(component_id)
    }

    async fn external_source_count(&self, id: ComponentId) -> ComponentResult<usize> {
        self.working_copy().await.external_source_count(id)
    }
}

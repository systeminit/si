use std::collections::VecDeque;

use async_trait::async_trait;
use si_id::{
    AttributePrototypeId,
    AttributeValueId,
};
use ulid::Ulid;

use crate::{
    DalContext,
    attribute::value::AttributeValueResult,
    workspace_snapshot::{
        WorkspaceSnapshotResult,
        edge_weight::EdgeWeightKindDiscriminants,
        graph::traits::attribute_value::AttributeValueExt as _,
        split_snapshot::SplitSnapshot,
        traits::attribute_value::AttributeValueExt,
    },
};

#[async_trait]
impl AttributeValueExt for SplitSnapshot {
    async fn attribute_value_view(
        &self,
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        //
        todo!()
    }

    async fn component_prototype_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>> {
        self.working_copy()
            .await
            .component_prototype_id(attribute_value_id)
    }
}

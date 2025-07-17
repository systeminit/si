use async_trait::async_trait;
use si_id::StaticArgumentValueId;

use crate::{
    DalContext,
    WorkspaceSnapshotError,
    attribute::prototype::argument::AttributePrototypeArgumentResult,
    layer_db_types::StaticArgumentValueContent,
    workspace_snapshot::{
        split_snapshot::SplitSnapshot,
        traits::static_argument_value::StaticArgumentValueExt,
    },
};

#[async_trait]
impl StaticArgumentValueExt for SplitSnapshot {
    async fn static_argument(
        &self,
        ctx: &DalContext,
        static_argument_value_id: StaticArgumentValueId,
    ) -> AttributePrototypeArgumentResult<StaticArgumentValueContent> {
        let content_hash = self
            .get_node_weight(static_argument_value_id)
            .await?
            .content_hash();
        ctx.layer_db()
            .cas()
            .try_read_as(&content_hash)
            .await?
            .ok_or_else(|| {
                WorkspaceSnapshotError::MissingContentFromStore(static_argument_value_id.into())
                    .into()
            })
    }

    async fn static_argument_value(
        &self,
        ctx: &DalContext,
        static_argument_value_id: StaticArgumentValueId,
    ) -> AttributePrototypeArgumentResult<serde_json::Value> {
        let StaticArgumentValueContent::V1(static_argument_content) =
            self.static_argument(ctx, static_argument_value_id).await?;
        Ok(static_argument_content.value.into())
    }
}

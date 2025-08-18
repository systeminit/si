use std::sync::Arc;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
};
pub use si_id::StaticArgumentValueId;

use super::AttributePrototypeArgumentResult;
use crate::{
    DalContext,
    layer_db_types::{
        StaticArgumentValueContent,
        StaticArgumentValueContentV1,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        content_address::ContentAddress,
        node_weight::NodeWeight,
    },
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct StaticArgumentValue {
    pub id: StaticArgumentValueId,
    pub timestamp: Timestamp,
    pub value: serde_json::Value,
}

impl StaticArgumentValue {
    pub fn assemble(id: StaticArgumentValueId, inner: StaticArgumentValueContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            value: inner.value.into(),
        }
    }

    pub fn id(&self) -> StaticArgumentValueId {
        self.id
    }

    pub async fn new(
        ctx: &DalContext,
        value: serde_json::Value,
    ) -> AttributePrototypeArgumentResult<Self> {
        let timestamp = Timestamp::now();
        let content = StaticArgumentValueContentV1 {
            timestamp,
            value: value.into(),
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(StaticArgumentValueContent::V1(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let node_weight =
            NodeWeight::new_content(id, lineage_id, ContentAddress::StaticArgumentValue(hash));

        ctx.workspace_snapshot()?
            .add_or_replace_node(node_weight)
            .await?;

        Ok(StaticArgumentValue::assemble(id.into(), content))
    }

    pub async fn get_by_id(
        ctx: &DalContext,
        id: StaticArgumentValueId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let hash = Self::value_content_hash(ctx, id).await?;

        let content: StaticArgumentValueContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let StaticArgumentValueContent::V1(inner) = content;

        Ok(StaticArgumentValue::assemble(id, inner))
    }

    pub async fn value_content_hash(
        ctx: &DalContext,
        id: StaticArgumentValueId,
    ) -> AttributePrototypeArgumentResult<ContentHash> {
        Ok(ctx
            .workspace_snapshot()?
            .get_node_weight(id)
            .await?
            .content_hash())
    }

    /// Get the value, formatted for debugging/display.
    pub async fn fmt_title(ctx: &DalContext, id: StaticArgumentValueId) -> String {
        Self::fmt_title_fallible(ctx, id)
            .await
            .unwrap_or_else(|e| e.to_string())
    }

    async fn fmt_title_fallible(
        ctx: &DalContext,
        id: StaticArgumentValueId,
    ) -> AttributePrototypeArgumentResult<String> {
        Ok(format!("{}", Self::get_by_id(ctx, id).await?.value))
    }
}

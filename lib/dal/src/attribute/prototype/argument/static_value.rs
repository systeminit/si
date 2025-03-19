use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    layer_db_types::{StaticArgumentValueContent, StaticArgumentValueContentV1},
    workspace_snapshot::{
        content_address::ContentAddress, node_weight::NodeWeight, WorkspaceSnapshotError,
    },
    DalContext, Timestamp,
};

use super::AttributePrototypeArgumentResult;

pub use si_id::StaticArgumentValueId;

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
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let ulid: si_events::ulid::Ulid = id.into();
        let node_weight = workspace_snapshot.get_node_weight(ulid).await?;
        let hash = node_weight.content_hash();

        let content: StaticArgumentValueContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let StaticArgumentValueContent::V1(inner) = content;

        Ok(StaticArgumentValue::assemble(id, inner))
    }
}

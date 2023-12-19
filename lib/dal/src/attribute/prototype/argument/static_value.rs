use content_store::Store;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use crate::{
    pk,
    workspace_snapshot::{
        content_address::ContentAddress, node_weight::NodeWeight, WorkspaceSnapshotError,
    },
    DalContext, Timestamp,
};

use super::AttributePrototypeArgumentResult;

pk!(StaticArgumentValueId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct StaticArgumentValue {
    pub id: StaticArgumentValueId,
    pub timestamp: Timestamp,
    pub value: serde_json::Value,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq, Debug)]
pub enum StaticArgumentValueContent {
    V1(StaticArgumentValueContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StaticArgumentValueContentV1 {
    pub timestamp: Timestamp,
    pub value: content_store::Value,
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

    pub fn new(
        ctx: &DalContext,
        value: serde_json::Value,
    ) -> AttributePrototypeArgumentResult<Self> {
        let timestamp = Timestamp::now();
        let content = StaticArgumentValueContentV1 {
            timestamp,
            value: value.into(),
        };

        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&StaticArgumentValueContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::StaticArgumentValue(hash))?;

        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
            workspace_snapshot.add_node(node_weight)?;
        }

        Ok(StaticArgumentValue::assemble(id.into(), content))
    }

    pub async fn get_by_id(
        ctx: &DalContext,
        id: StaticArgumentValueId,
    ) -> AttributePrototypeArgumentResult<Self> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let ulid: ulid::Ulid = id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(ulid)?;
        let node_weight = workspace_snapshot.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: StaticArgumentValueContent = ctx
            .content_store()
            .try_lock()?
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let StaticArgumentValueContent::V1(inner) = content;

        Ok(StaticArgumentValue::assemble(id, inner))
    }
}

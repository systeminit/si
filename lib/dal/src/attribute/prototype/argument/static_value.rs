use content_store::Store;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::{
    change_set_pointer::ChangeSetPointerError,
    func::argument::FuncArgumentId,
    impl_standard_model, pk,
    provider::internal::InternalProviderId,
    standard_model, standard_model_accessor,
    workspace_snapshot::{
        content_address::{ContentAddress, ContentAddressDiscriminants},
        edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants},
        node_weight::{ContentNodeWeight, NodeWeight, NodeWeightError},
        WorkspaceSnapshotError,
    },
    AttributePrototypeId, ComponentId, DalContext, ExternalProviderId, HistoryEventError,
    StandardModel, StandardModelError, Tenancy, Timestamp, TransactionsError, Visibility,
};

use super::{
    AttributePrototypeArgumentError, AttributePrototypeArgumentId, AttributePrototypeArgumentResult,
};

pk!(StaticArgumentValueId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct StaticArgumentValue {
    pub id: StaticArgumentValueId,
    pub timestamp: Timestamp,
    pub value: serde_json::Value,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum StaticArgumentValueContent {
    V1(StaticArgumentValueContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StaticArgumentValueContentV1 {
    pub timestamp: Timestamp,
    pub value: serde_json::Value,
}

impl StaticArgumentValue {
    pub fn assemble(id: StaticArgumentValueId, inner: StaticArgumentValueContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            value: inner.value,
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
        let content = StaticArgumentValueContentV1 { timestamp, value };

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
}

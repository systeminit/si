use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use thiserror::Error;

use crate::component::ComponentCreatedPayload;
use crate::{
    pkg::ModuleImported, user::CursorPayload, ChangeSetPk, DalContext, PropId, StandardModelError,
    TransactionsError, WorkspacePk,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WsEventError {
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("no user in context")]
    NoUserInContext,
    #[error("no workspace in tenancy")]
    NoWorkspaceInTenancy,
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type WsEventResult<T> = Result<T, WsEventError>;

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "kind", content = "data")]
#[allow(clippy::large_enum_variant)]
pub enum WsPayload {
    ChangeSetApplied(ChangeSetPk),
    ChangeSetCanceled(ChangeSetPk),
    ChangeSetCreated(ChangeSetPk),
    ChangeSetWritten(ChangeSetPk),
    // CheckedQualifications(QualificationCheckPayload),
    // CodeGenerated(CodeGeneratedPayload),
    ComponentCreated(ComponentCreatedPayload),
    Cursor(CursorPayload),
    // FixBatchReturn(FixBatchReturn),
    // FixReturn(FixReturn),
    // LogLine(LogLinePayload),
    ModuleImported(ModuleImported),
    // ResourceRefreshed(ResourceRefreshedPayload),
    // SchemaCreated(SchemaPk),
    // StatusUpdate(StatusMessage),
}

#[remain::sorted]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Copy, Hash)]
#[serde(rename_all = "camelCase", tag = "kind", content = "id")]
pub enum StatusValueKind {
    Attribute(PropId),
    CodeGen,
    // InputSocket(SocketId),
    Internal,
    // OutputSocket(SocketId),
    Qualification,
}

// #[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
// #[serde(rename_all = "camelCase")]
// pub struct AttributeValueStatusUpdate {
//     value_id: AttributeValueId,
//     component_id: ComponentId,
//     value_kind: StatusValueKind,
// }

// impl AttributeValueStatusUpdate {
//     pub fn new(
//         value_id: AttributeValueId,
//         component_id: ComponentId,
//         value_kind: StatusValueKind,
//     ) -> Self {
//         Self {
//             value_id,
//             component_id,
//             value_kind,
//         }
//     }
// }

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct WsEvent {
    version: i64,
    workspace_pk: WorkspacePk,
    change_set_pk: ChangeSetPk,
    payload: WsPayload,
}

impl WsEvent {
    pub async fn new_raw(
        workspace_pk: WorkspacePk,
        change_set_pk: ChangeSetPk,
        payload: WsPayload,
    ) -> WsEventResult<Self> {
        Ok(WsEvent {
            version: 1,
            workspace_pk,
            change_set_pk,
            payload,
        })
    }
    pub async fn new(ctx: &DalContext, payload: WsPayload) -> WsEventResult<Self> {
        let workspace_pk = match ctx.tenancy().workspace_pk() {
            Some(pk) => pk,
            None => {
                return Err(WsEventError::NoWorkspaceInTenancy);
            }
        };
        let change_set_pk = ctx.visibility().change_set_pk;
        Self::new_raw(workspace_pk, change_set_pk, payload).await
    }

    pub fn workspace_pk(&self) -> WorkspacePk {
        self.workspace_pk
    }

    /// Publishes the [`event`](Self) to the [`NatsTxn`](si_data_nats::NatsTxn). When the
    /// transaction is committed, the [`event`](Self) will be published for external use.
    pub async fn publish_on_commit(&self, ctx: &DalContext) -> WsEventResult<()> {
        let subject = format!("si.workspace_pk.{}.event", self.workspace_pk);
        ctx.txns().await?.nats().publish(subject, &self).await?;
        Ok(())
    }
}

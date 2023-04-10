use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use thiserror::Error;

use crate::component::confirmation::ConfirmationsUpdatedPayload;
use crate::component::ComponentCreatedPayload;
use crate::{
    component::{code::CodeGeneratedPayload, resource::ResourceRefreshedPayload},
    fix::{batch::FixBatchReturn, FixReturn},
    qualification::QualificationCheckPayload,
    status::StatusMessage,
    workflow::CommandOutput,
    AttributeValueId, ChangeSetPk, ComponentId, DalContext, PropId, SchemaPk, SocketId,
    StandardModelError, TransactionsError, WorkspacePk,
};

#[derive(Error, Debug)]
pub enum WsEventError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("no workspace in tenancy")]
    NoWorkspaceInTenancy,
}

pub type WsEventResult<T> = Result<T, WsEventError>;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "kind", content = "data")]
#[allow(clippy::large_enum_variant)]
pub enum WsPayload {
    ChangeSetCreated(ChangeSetPk),
    ChangeSetApplied(ChangeSetPk),
    ChangeSetCanceled(ChangeSetPk),
    ChangeSetWritten(ChangeSetPk),
    ComponentCreated(ComponentCreatedPayload),
    SchemaCreated(SchemaPk),
    ResourceRefreshed(ResourceRefreshedPayload),
    ConfirmationsUpdated(ConfirmationsUpdatedPayload),
    CheckedQualifications(QualificationCheckPayload),
    CommandOutput(CommandOutput),
    CodeGenerated(CodeGeneratedPayload),
    FixBatchReturn(FixBatchReturn),
    FixReturn(FixReturn),
    StatusUpdate(StatusMessage),
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Copy, Hash)]
#[serde(rename_all = "camelCase", tag = "kind", content = "id")]
pub enum StatusValueKind {
    Attribute(PropId),
    CodeGen,
    Qualification,
    Internal,
    InputSocket(SocketId),
    OutputSocket(SocketId),
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValueStatusUpdate {
    value_id: AttributeValueId,
    component_id: ComponentId,
    value_kind: StatusValueKind,
}

impl AttributeValueStatusUpdate {
    pub fn new(
        value_id: AttributeValueId,
        component_id: ComponentId,
        value_kind: StatusValueKind,
    ) -> Self {
        Self {
            value_id,
            component_id,
            value_kind,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct WsEvent {
    version: i64,
    workspace_pk: WorkspacePk,
    change_set_pk: ChangeSetPk,
    payload: WsPayload,
}

impl WsEvent {
    pub async fn new(ctx: &DalContext, payload: WsPayload) -> WsEventResult<Self> {
        let workspace_pk = match ctx.tenancy().workspace_pk() {
            Some(pk) => pk,
            None => {
                return Err(WsEventError::NoWorkspaceInTenancy);
            }
        };
        let change_set_pk = ctx.visibility().change_set_pk;

        Ok(WsEvent {
            version: 1,
            workspace_pk,
            change_set_pk,
            payload,
        })
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

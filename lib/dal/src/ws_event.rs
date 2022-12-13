use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use thiserror::Error;

use crate::component::code::CodeGeneratedPayload;
use crate::component::resource::ResourceRefreshId;
use crate::confirmation_status::ConfirmationStatusUpdate;
use crate::fix::batch::FixBatchReturn;
use crate::fix::FixReturn;
use crate::qualification::QualificationCheckPayload;
use crate::status::StatusMessage;
use crate::workflow::{CommandOutput, CommandReturn};
use crate::{
    AttributeValueId, BillingAccountId, ChangeSetPk, ComponentId, ConfirmationPrototypeError,
    DalContext, HistoryActor, PropId, ReadTenancy, SchemaPk, SocketId, StandardModelError,
};

#[derive(Error, Debug)]
pub enum WsEventError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error(transparent)]
    ConfirmationPrototype(#[from] Box<ConfirmationPrototypeError>),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
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
    SchemaCreated(SchemaPk),
    ResourceRefreshed(ResourceRefreshId),
    CheckedQualifications(QualificationCheckPayload),
    CommandOutput(CommandOutput),
    CodeGenerated(CodeGeneratedPayload),
    CommandReturn(CommandReturn),
    FixBatchReturn(FixBatchReturn),
    FixReturn(FixReturn),
    ConfirmationStatusUpdate(ConfirmationStatusUpdate),
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
    billing_account_ids: Vec<BillingAccountId>,
    history_actor: HistoryActor,
    change_set_pk: ChangeSetPk,
    payload: WsPayload,
}

impl WsEvent {
    pub fn new(ctx: &DalContext, payload: WsPayload) -> Self {
        let billing_account_ids = Self::billing_account_id_from_tenancy(ctx.read_tenancy());
        let history_actor = *ctx.history_actor();
        let change_set_pk = ctx.visibility().change_set_pk;

        WsEvent {
            version: 1,
            billing_account_ids,
            history_actor,
            change_set_pk,
            payload,
        }
    }

    pub fn billing_account_id_from_tenancy(tenancy: &ReadTenancy) -> Vec<BillingAccountId> {
        tenancy.billing_accounts().into()
    }

    pub async fn publish(&self, ctx: &DalContext) -> WsEventResult<()> {
        for billing_account_id in self.billing_account_ids.iter() {
            let subject = format!("si.billing_account_id.{}.event", billing_account_id);
            ctx.nats_txn().publish(subject, &self).await?;
        }
        Ok(())
    }

    pub async fn publish_immediately(&self, ctx: &DalContext) -> WsEventResult<()> {
        for billing_account_id in self.billing_account_ids.iter() {
            let subject = format!("si.billing_account_id.{}.event", billing_account_id);
            let msg_bytes = serde_json::to_vec(self)?;
            ctx.nats_conn().publish(subject, msg_bytes).await?;
        }
        Ok(())
    }
}

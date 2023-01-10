use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use thiserror::Error;

use crate::component::confirmation::ConfirmationRunPayload;
use crate::{
    component::{code::CodeGeneratedPayload, resource::ResourceRefreshId},
    confirmation_status::ConfirmationStatusUpdate,
    fix::{batch::FixBatchReturn, FixReturn},
    qualification::QualificationCheckPayload,
    status::StatusMessage,
    workflow::{CommandOutput, CommandReturn},
    ActorView, AttributeValueId, BillingAccountId, ChangeSetPk, ComponentId,
    ConfirmationPrototypeError, DalContext, PropId, ReadTenancy, SchemaPk, SocketId,
    StandardModelError,
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
    RanConfirmations(ConfirmationRunPayload),
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
    actor: ActorView,
    change_set_pk: ChangeSetPk,
    payload: WsPayload,
}

impl WsEvent {
    pub async fn new(ctx: &DalContext, payload: WsPayload) -> WsEventResult<Self> {
        let billing_account_ids = Self::billing_account_id_from_tenancy(ctx.read_tenancy());
        let change_set_pk = ctx.visibility().change_set_pk;
        let actor = ActorView::from_history_actor(ctx, *ctx.history_actor()).await?;

        Ok(WsEvent {
            version: 1,
            billing_account_ids,
            actor,
            change_set_pk,
            payload,
        })
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

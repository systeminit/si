use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use thiserror::Error;

use crate::component::ResourceRefreshId;
use crate::confirmation_status::ConfirmationStatusUpdate;
use crate::fix::batch::FixBatchReturn;
use crate::fix::FixReturn;
use crate::qualification::QualificationCheckId;
use crate::schema::variant::leaves::code_generation::CodeGeneratedPayload;
use crate::workflow::{CommandOutput, CommandReturn};
use crate::{
    BillingAccountId, ChangeSetPk, ConfirmationPrototypeError, DalContext, HistoryActor,
    ReadTenancy, SchemaPk, StandardModelError,
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
    CheckedQualifications(QualificationCheckId),
    CommandOutput(CommandOutput),
    CodeGenerated(CodeGeneratedPayload),
    CommandReturn(CommandReturn),
    FixBatchReturn(FixBatchReturn),
    FixReturn(FixReturn),
    ConfirmationStatusUpdate(ConfirmationStatusUpdate),
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct WsEvent {
    version: i64,
    billing_account_ids: Vec<BillingAccountId>,
    history_actor: HistoryActor,
    payload: WsPayload,
}

impl WsEvent {
    pub fn new(ctx: &DalContext, payload: WsPayload) -> Self {
        let billing_account_ids = Self::billing_account_id_from_tenancy(ctx.read_tenancy());
        let history_actor = ctx.history_actor().clone();
        WsEvent {
            version: 1,
            billing_account_ids,
            history_actor,
            payload,
        }
    }

    pub fn new_raw(
        billing_account_ids: Vec<BillingAccountId>,
        history_actor: HistoryActor,
        payload: WsPayload,
    ) -> Self {
        WsEvent {
            version: 1,
            billing_account_ids,
            history_actor,
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
}

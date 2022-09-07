use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::NatsError;

use crate::attribute::value::DependentValuesUpdated;
use crate::code_generation_resolver::CodeGenerationId;
use crate::qualification::QualificationCheckId;
use crate::resource::ResourceSyncId;
use crate::workflow::CommandOutput;
use crate::{BillingAccountId, ChangeSetPk, DalContext, HistoryActor, ReadTenancy, SchemaPk};

#[derive(Error, Debug)]
pub enum WsEventError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
}

pub type WsEventResult<T> = Result<T, WsEventError>;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "kind", content = "data")]
pub enum WsPayload {
    ChangeSetCreated(ChangeSetPk),
    ChangeSetApplied(ChangeSetPk),
    ChangeSetCanceled(ChangeSetPk),
    ChangeSetWritten(ChangeSetPk),
    SchemaCreated(SchemaPk),
    ResourceSynced(ResourceSyncId),
    CodeGenerated(CodeGenerationId),
    CheckedQualifications(QualificationCheckId),
    UpdatedDependentValue(DependentValuesUpdated),
    CommandOutput(CommandOutput),
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

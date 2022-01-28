use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::{NatsError, NatsTxn};

use crate::resource::ResourceSyncId;
use crate::{BillingAccountId, ChangeSetPk, HistoryActor, SchemaPk, Tenancy};

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
    EditSessionSaved(ChangeSetPk),
    SchemaCreated(SchemaPk),
    ResourceSynced(ResourceSyncId),
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct WsEvent {
    version: i64,
    billing_account_ids: Vec<BillingAccountId>,
    history_actor: HistoryActor,
    payload: WsPayload,
}

impl WsEvent {
    pub fn new(
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

    pub fn billing_account_id_from_tenancy(tenancy: &Tenancy) -> Vec<BillingAccountId> {
        tenancy.billing_account_ids.clone()
    }

    pub async fn publish(&self, nats: &NatsTxn) -> WsEventResult<()> {
        for billing_account_id in self.billing_account_ids.iter() {
            let subject = format!("si.billing_account_id.{}.event", billing_account_id);
            nats.publish(subject, &self).await?;
        }
        Ok(())
    }
}

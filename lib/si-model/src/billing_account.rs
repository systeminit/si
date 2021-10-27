use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, HistoryActor, HistoryEvent,
    HistoryEventError, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgError, PgTxn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BillingAccountError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type BillingAccountResult<T> = Result<T, BillingAccountError>;

pk!(BillingAccountPk);
pk!(BillingAccountId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BillingAccount {
    pk: BillingAccountPk,
    id: BillingAccountId,
    name: String,
    description: Option<String>,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: BillingAccount,
    pk: BillingAccountPk,
    id: BillingAccountId,
    table_name: "billing_accounts",
    history_event_label_base: "billing_account",
    history_event_message_name: "Billing Account"
}

impl BillingAccount {
    #[tracing::instrument(skip(txn, nats, name, description))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        description: Option<&String>,
    ) -> BillingAccountResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM billing_account_create_v1($1, $2, $3, $4)",
                &[&tenancy, &visibility, &name, &description],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let _history_event = HistoryEvent::new(
            &txn,
            &nats,
            Self::history_event_label(vec!["create"]),
            &history_actor,
            Self::history_event_message("created"),
            &serde_json::json![{ "object": json, "visibility": &visibility }],
            &tenancy,
        )
        .await?;
        let object: Self = serde_json::from_value(json)?;
        Ok(object)
    }

    standard_model_accessor!(name, String);
    standard_model_accessor!(description, Option<String>);
}

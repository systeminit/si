use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    BillingAccount, BillingAccountId, HistoryActor, HistoryEventError, StandardModel,
    StandardModelError, Tenancy, Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum OrganizationError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
}

pub type OrganizationResult<T> = Result<T, OrganizationError>;

pk!(OrganizationPk);
pk!(OrganizationId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Organization {
    pk: OrganizationPk,
    id: OrganizationId,
    name: String,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Organization,
    pk: OrganizationPk,
    id: OrganizationId,
    table_name: "organizations",
    history_event_label_base: "organization",
    history_event_message_name: "Organization"
}

impl Organization {
    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
    ) -> OrganizationResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM organization_create_v1($1, $2, $3)",
                &[&tenancy, &visibility, &name],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            &txn,
            &nats,
            &tenancy,
            &visibility,
            &history_actor,
            row,
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, OrganizationResult);
    standard_model_belongs_to!(
        lookup_fn: billing_account,
        set_fn: set_billing_account,
        unset_fn: unset_billing_account,
        table: "organization_belongs_to_billing_account",
        model_table: "billing_accounts",
        belongs_to_id: BillingAccountId,
        returns: BillingAccount,
        result: OrganizationResult,
    );
}

use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, BillingAccount,
    BillingAccountError, BillingAccountPk, DalContext, HistoryEventError, StandardModel,
    StandardModelError, Timestamp, Visibility, WriteTenancy,
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
    #[error(transparent)]
    BillingAccount(#[from] Box<BillingAccountError>),
}

pub type OrganizationResult<T> = Result<T, OrganizationError>;

pk!(OrganizationPk);
pk!(OrganizationId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Organization {
    pk: OrganizationPk,
    id: OrganizationId,
    name: String,
    billing_account_pk: BillingAccountPk,
    #[serde(flatten)]
    tenancy: WriteTenancy,
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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        billing_account_pk: BillingAccountPk,
    ) -> OrganizationResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM organization_create_v1($1, $2, $3, $4)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &billing_account_pk,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, OrganizationResult);
    standard_model_accessor!(billing_account_pk, Pk(BillingAccountPk), OrganizationResult);

    pub async fn billing_account(&self, ctx: &DalContext) -> OrganizationResult<BillingAccount> {
        Ok(BillingAccount::get_by_pk(ctx, &self.billing_account_pk)
            .await
            .map_err(Box::new)?)
    }
}

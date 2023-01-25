use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    pk, standard_model_accessor_ro, BillingAccount, BillingAccountError, BillingAccountPk,
    DalContext, HistoryEvent, HistoryEventError, Timestamp, TransactionsError,
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
    #[error(transparent)]
    BillingAccount(#[from] Box<BillingAccountError>),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type OrganizationResult<T> = Result<T, OrganizationError>;

pk!(OrganizationPk);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Organization {
    pk: OrganizationPk,
    name: String,
    billing_account_pk: BillingAccountPk,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl Organization {
    pub fn pk(&self) -> &OrganizationPk {
        &self.pk
    }

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
                "SELECT object FROM organization_create_v1($1, $2)",
                &[&name, &billing_account_pk],
            )
            .await?;

        // Inlined `finish_create_from_row`

        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;

        // Ensures HistoryEvent gets stored in our billing account
        let ctx = ctx.clone_with_new_organization_tenancies(object.pk).await?;
        let _history_event = HistoryEvent::new(
            &ctx,
            "organization.create".to_owned(),
            "Organization created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor_ro!(name, String);
    standard_model_accessor_ro!(billing_account_pk, BillingAccountPk);

    pub async fn billing_account(&self, ctx: &DalContext) -> OrganizationResult<BillingAccount> {
        Ok(BillingAccount::get_by_pk(ctx, &self.billing_account_pk)
            .await
            .map_err(Box::new)?)
    }
}

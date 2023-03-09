use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    pk, standard_model, standard_model_accessor_ro, BillingAccountPk, DalContext, HistoryEvent,
    HistoryEventError, StandardModelError, Tenancy, Timestamp, TransactionsError,
};

const WORKSPACE_GET_BY_PK: &str = include_str!("queries/workspace/get_by_pk.sql");

#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    Transactions(#[from] Box<TransactionsError>),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

pk!(WorkspacePk);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pk: WorkspacePk,
    billing_account_pk: BillingAccountPk,
    name: String,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl Workspace {
    pub fn pk(&self) -> &WorkspacePk {
        &self.pk
    }

    #[instrument(skip_all)]
    pub async fn builtin(ctx: &DalContext) -> WorkspaceResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM workspace_find_or_create_builtin_v1()",
                &[],
            )
            .await?;

        let object = standard_model::object_from_row(row)?;
        Ok(object)
    }

    #[instrument(skip_all)]
    pub async fn new(
        ctx: &mut DalContext,
        name: impl AsRef<str>,
        billing_account_pk: BillingAccountPk,
    ) -> WorkspaceResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM workspace_create_v1($1, $2)",
                &[&name, &billing_account_pk],
            )
            .await?;

        // Inlined `finish_create_from_row`

        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;

        ctx.update_tenancy(Tenancy::new(object.pk));

        let _history_event = HistoryEvent::new(
            ctx,
            "workspace.create".to_owned(),
            "Workspace created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;
        Ok(object)
    }

    pub async fn get_by_pk(ctx: &DalContext, pk: &WorkspacePk) -> WorkspaceResult<Workspace> {
        let row = ctx
            .txns()
            .pg()
            .query_one(WORKSPACE_GET_BY_PK, &[&pk])
            .await?;
        let result = standard_model::object_from_row(row)?;
        Ok(result)
    }

    standard_model_accessor_ro!(name, String);
    standard_model_accessor_ro!(billing_account_pk, BillingAccountPk);
}

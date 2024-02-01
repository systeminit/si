//! Contains the ability to resolve _current_ fixes, provided by
//! [`FixResolver`](crate::FixResolver).

use crate::{DalContext, FixId, TransactionsError};
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, ActionPrototypeId,
    HistoryEventError, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FixResolverError {
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type FixResolverResult<T> = Result<T, FixResolverError>;

pk!(FixResolverPk);
pk!(FixResolverId);

/// Determines what "fix" to run for a given `Action`
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FixResolver {
    pk: FixResolverPk,
    id: FixResolverId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
    /// The "fix" to run.
    action_prototype_id: ActionPrototypeId,
    /// The ternary state of a "fix" execution.
    success: Option<bool>,
    /// Indicates the last [`Fix`](crate::Fix) that was ran corresponding to this
    /// [`resolver`](Self).
    last_fix_id: FixId,
}

impl_standard_model! {
    model: FixResolver,
    pk: FixResolverPk,
    id: FixResolverId,
    table_name: "fix_resolvers",
    history_event_label_base: "fix_resolver",
    history_event_message_name: "Fix Resolver"
}

impl FixResolver {
    /// Private constructor method for creating a [`FixResolver`]. Use [`Self::upsert()`] instead.
    async fn new(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        success: Option<bool>,
        last_fix_id: FixId,
    ) -> FixResolverResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM fix_resolver_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &action_prototype_id,
                    &success,
                    &last_fix_id,
                ],
            )
            .await?;

        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    /// Find or create a new [`resolver`](Self) based on the information provided.
    pub async fn upsert(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        success: Option<bool>,
        last_fix_id: FixId,
    ) -> FixResolverResult<Self> {
        Self::new(ctx, action_prototype_id, success, last_fix_id).await
    }

    standard_model_accessor!(
        action_prototype_id,
        Pk(ActionPrototypeId),
        FixResolverResult
    );
    standard_model_accessor!(success, Option<bool>, FixResolverResult);
    standard_model_accessor!(last_fix_id, Pk(FixId), FixResolverResult);
}

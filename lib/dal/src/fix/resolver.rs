//! Contains the ability to resolve _current_ fixes, provided by
//! [`FixResolver`](crate::FixResolver).

use crate::{AttributeValueId, DalContext, FixId, TransactionsError};
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, HistoryEventError,
    StandardModel, StandardModelError, Tenancy, Timestamp, Visibility, WorkflowPrototypeId,
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

const FIND_FOR_CONFIRMATION_ATTRIBUTE_VALUE: &str =
    include_str!("../queries/fix_resolver_find_for_confirmation_attribute_value.sql");

pk!(FixResolverPk);
pk!(FixResolverId);

/// Determines what "fix" to run for a given [`"confirmation"`](crate::schema::variant::leaves)
/// [`AttributeValueId`](crate::AttributeValue).
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
    workflow_prototype_id: WorkflowPrototypeId,
    /// Corresponds to the [`AttributeValue`](crate::AttributeValue) corresponding to the
    attribute_value_id: AttributeValueId,
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
    #[instrument(skip_all)]
    async fn new(
        ctx: &DalContext,
        workflow_prototype_id: WorkflowPrototypeId,
        attribute_value_id: AttributeValueId,
        success: Option<bool>,
        last_fix_id: FixId,
    ) -> FixResolverResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM fix_resolver_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &workflow_prototype_id,
                    &attribute_value_id,
                    &success,
                    &last_fix_id,
                ],
            )
            .await?;

        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    /// Find [`self`](Self) for a given [`AttributeValueId`](crate::AttributeValue).
    pub async fn find_for_confirmation_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> FixResolverResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_FOR_CONFIRMATION_ATTRIBUTE_VALUE,
                &[ctx.tenancy(), ctx.visibility(), &attribute_value_id],
            )
            .await?;
        let object = standard_model::option_object_from_row(row)?;
        Ok(object)
    }

    /// Find or create a new [`resolver`](Self) based on the information provided.
    pub async fn upsert(
        ctx: &DalContext,
        workflow_prototype_id: WorkflowPrototypeId,
        attribute_value_id: AttributeValueId,
        success: Option<bool>,
        last_fix_id: FixId,
    ) -> FixResolverResult<Self> {
        if let Some(mut resolver) =
            Self::find_for_confirmation_attribute_value(ctx, attribute_value_id).await?
        {
            resolver
                .set_workflow_prototype_id(ctx, workflow_prototype_id)
                .await?;
            resolver.set_success(ctx, success).await?;
            resolver.set_last_fix_id(ctx, last_fix_id).await?;
            Ok(resolver)
        } else {
            Ok(Self::new(
                ctx,
                workflow_prototype_id,
                attribute_value_id,
                success,
                last_fix_id,
            )
            .await?)
        }
    }

    standard_model_accessor!(
        workflow_prototype_id,
        Pk(WorkflowPrototypeId),
        FixResolverResult
    );
    standard_model_accessor!(attribute_value_id, Pk(AttributeValueId), FixResolverResult);
    standard_model_accessor!(success, Option<bool>, FixResolverResult);
    standard_model_accessor!(last_fix_id, Pk(FixId), FixResolverResult);
}

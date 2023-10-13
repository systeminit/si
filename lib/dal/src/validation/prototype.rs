use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::validation::prototype::context::ValidationPrototypeContextBuilder;
use crate::{
    func::FuncId,
    impl_standard_model, pk,
    standard_model::{self, objects_from_rows},
    standard_model_accessor, DalContext, HistoryEventError, Prop, PropId, SchemaVariantId,
    StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
};
use crate::{PropKind, SchemaId, TransactionsError, ValidationPrototypeContext};

pub mod context;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValidationPrototypeError {
    #[error("prop for validation prototype context is not of primitive prop kind, found: {0:?}")]
    ContextPropKindIsNotPrimitive(PropKind),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("for builder {0:?}, the following fields must be set: {1:?}")]
    PrerequisteFieldsUnset(ValidationPrototypeContextBuilder, Vec<&'static str>),
    #[error("prop not found by id: {0}")]
    PropNotFound(PropId),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type ValidationPrototypeResult<T> = Result<T, ValidationPrototypeError>;

const LIST_FOR_PROP: &str = include_str!("../queries/validation_prototype/list_for_prop.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("../queries/validation_prototype/list_for_schema_variant.sql");
const LIST_FOR_FUNC: &str = include_str!("../queries/validation_prototype/list_for_func.sql");
const FIND_FOR_CONTEXT: &str = include_str!("../queries/validation_prototype/find_for_context.sql");

pk!(ValidationPrototypePk);
pk!(ValidationPrototypeId);

// An ValidationPrototype joins a `Func` to the context in which
// the component that is created with it can use to generate a ValidationResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationPrototype {
    pk: ValidationPrototypePk,
    id: ValidationPrototypeId,
    func_id: FuncId,
    args: serde_json::Value,
    link: Option<String>,
    prop_id: PropId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: ValidationPrototype,
    pk: ValidationPrototypePk,
    id: ValidationPrototypeId,
    table_name: "validation_prototypes",
    history_event_label_base: "validation_prototype",
    history_event_message_name: "Validation Prototype"
}

impl ValidationPrototype {
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        func_id: FuncId,
        args: serde_json::Value,
        context: ValidationPrototypeContext,
    ) -> ValidationPrototypeResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM validation_prototype_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &args,
                    &context.prop_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(func_id, Pk(FuncId), ValidationPrototypeResult);
    standard_model_accessor!(args, Json<JsonValue>, ValidationPrototypeResult);
    standard_model_accessor!(link, Option<String>, ValidationPrototypeResult);
    standard_model_accessor!(prop_id, Pk(PropId), ValidationPrototypeResult);
    standard_model_accessor!(schema_id, Pk(SchemaId), ValidationPrototypeResult);
    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        ValidationPrototypeResult
    );

    pub fn context(&self) -> ValidationPrototypeContext {
        ValidationPrototypeContext::new_unchecked(
            self.prop_id,
            self.schema_variant_id,
            self.schema_id,
        )
    }

    /// List all [`ValidationPrototypes`](Self) for a given [`Prop`](crate::Prop).
    #[instrument(skip_all)]
    pub async fn list_for_prop(
        ctx: &DalContext,
        prop_id: PropId,
    ) -> ValidationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(LIST_FOR_PROP, &[ctx.tenancy(), ctx.visibility(), &prop_id])
            .await?;
        let object = objects_from_rows(rows)?;
        Ok(object)
    }

    /// List all [`ValidationPrototypes`](Self) for all [`Props`](crate::Prop) in a
    /// [`SchemaVariant`](crate::SchemaVariant).
    ///
    /// _You can access the [`PropId`](crate::Prop) via the [`ValidationPrototypeContext`], if
    /// needed._
    #[instrument(skip_all)]
    pub async fn list_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ValidationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_SCHEMA_VARIANT,
                &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;
        let object = objects_from_rows(rows)?;
        Ok(object)
    }

    /// List all [`ValidationPrototypes`](Self) for a [`Func`](crate::Func)
    #[instrument(skip_all)]
    pub async fn list_for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> ValidationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(LIST_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), &func_id])
            .await?;

        Ok(objects_from_rows(rows)?)
    }

    pub async fn find_for_context(
        ctx: &DalContext,
        context: ValidationPrototypeContext,
    ) -> ValidationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context.prop_id(),
                    &context.schema_variant_id(),
                    &context.schema_id(),
                ],
            )
            .await?;

        Ok(objects_from_rows(rows)?)
    }

    pub async fn prop(&self, ctx: &DalContext) -> ValidationPrototypeResult<Prop> {
        Prop::get_by_id(ctx, &self.prop_id())
            .await?
            .ok_or(ValidationPrototypeError::PropNotFound(self.prop_id()))
    }
}

use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use thiserror::Error;

use telemetry::prelude::*;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, ComponentId, DalContext,
    Func, FuncError, FuncId, HistoryEventError, SchemaVariantId, StandardModel, StandardModelError,
    Tenancy, Timestamp, TransactionsError, Visibility,
};

const FIND_FOR_CONTEXT: &str =
    include_str!("./queries/reconciliation_prototype_find_for_context.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ReconciliationPrototypeError {
    #[error("func: {0}")]
    Func(#[from] FuncError),
    #[error("history event: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("pg: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type ReconciliationPrototypeResult<T> = Result<T, ReconciliationPrototypeError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationPrototypeContext {
    pub component_id: ComponentId,
    pub schema_variant_id: SchemaVariantId,
}

impl ReconciliationPrototypeContext {
    pub fn new(context_field: impl Into<ReconciliationPrototypeContextField>) -> Self {
        match context_field.into() {
            ReconciliationPrototypeContextField::SchemaVariant(schema_variant_id) => {
                ReconciliationPrototypeContext {
                    component_id: ComponentId::NONE,
                    schema_variant_id,
                }
            }
            ReconciliationPrototypeContextField::Component(component_id) => {
                ReconciliationPrototypeContext {
                    component_id,
                    schema_variant_id: SchemaVariantId::NONE,
                }
            }
        }
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn set_component_id(&mut self, component_id: ComponentId) {
        self.component_id = component_id;
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) {
        self.schema_variant_id = schema_variant_id;
    }
}

pk!(ReconciliationPrototypePk);
pk!(ReconciliationPrototypeId);

// An ReconciliationPrototype joins a `WorkflowPrototype` to the context in which
// the component that is created with it can use to generate a ConfirmationResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationPrototype {
    pk: ReconciliationPrototypePk,
    id: ReconciliationPrototypeId,
    name: String,
    func_id: FuncId,
    component_id: ComponentId,
    schema_variant_id: SchemaVariantId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

#[remain::sorted]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReconciliationPrototypeContextField {
    Component(ComponentId),
    SchemaVariant(SchemaVariantId),
}

impl From<ComponentId> for ReconciliationPrototypeContextField {
    fn from(component_id: ComponentId) -> Self {
        ReconciliationPrototypeContextField::Component(component_id)
    }
}

impl From<SchemaVariantId> for ReconciliationPrototypeContextField {
    fn from(schema_variant_id: SchemaVariantId) -> Self {
        ReconciliationPrototypeContextField::SchemaVariant(schema_variant_id)
    }
}

impl_standard_model! {
    model: ReconciliationPrototype,
    pk: ReconciliationPrototypePk,
    id: ReconciliationPrototypeId,
    table_name: "reconciliation_prototypes",
    history_event_label_base: "reconciliation_prototype",
    history_event_message_name: "Reconciliation Prototype"
}

impl ReconciliationPrototype {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn upsert(
        ctx: &DalContext,
        func_id: FuncId,
        name: &str,
        context: ReconciliationPrototypeContext,
    ) -> ReconciliationPrototypeResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM reconciliation_prototype_upsert_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &name,
                    &context.component_id(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    pub async fn find_for_context(
        ctx: &DalContext,
        context: ReconciliationPrototypeContext,
    ) -> ReconciliationPrototypeResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context.component_id,
                    &context.schema_variant_id,
                ],
            )
            .await?;
        Ok(standard_model::object_option_from_row_option(row)?)
    }

    pub async fn func(&self, ctx: &DalContext) -> ReconciliationPrototypeResult<Func> {
        let func = Func::get_by_id(ctx, &self.func_id)
            .await?
            .ok_or(FuncError::NotFound(self.func_id))?;
        Ok(func)
    }

    standard_model_accessor!(func_id, Pk(FuncId), ReconciliationPrototypeResult);
    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        ReconciliationPrototypeResult
    );
    standard_model_accessor!(component_id, Pk(ComponentId), ReconciliationPrototypeResult);

    standard_model_accessor!(name, String, ReconciliationPrototypeResult);

    pub fn context(&self) -> ReconciliationPrototypeContext {
        let mut context = ReconciliationPrototypeContext::new(self.component_id);
        context.set_schema_variant_id(self.schema_variant_id);
        context
    }
}

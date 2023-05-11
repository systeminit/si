use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::FuncId, impl_standard_model, pk, standard_model, standard_model_accessor, ComponentId,
    DalContext, HistoryEventError, SchemaVariantId, StandardModel, StandardModelError, Tenancy,
    Timestamp, TransactionsError, Visibility,
};

const FIND_FOR_FUNC: &str = include_str!("queries/command_prototype/find_for_func.sql");
const FIND_FOR_FUNC_AND_SCHEMA_VARIANT: &str =
    include_str!("queries/command_prototype/find_for_func_and_schema_variant.sql");
const FIND_FOR_CONTEXT: &str = include_str!("queries/command_prototype/find_for_context.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum CommandPrototypeError {
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type CommandPrototypeResult<T> = Result<T, CommandPrototypeError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CommandPrototypeContext {
    pub component_id: ComponentId,
    pub schema_variant_id: SchemaVariantId,
}

impl Default for CommandPrototypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: ComponentId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
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

pk!(CommandPrototypePk);
pk!(CommandPrototypeId);

impl_standard_model! {
    model: CommandPrototype,
    pk: CommandPrototypePk,
    id: CommandPrototypeId,
    table_name: "command_prototypes",
    history_event_label_base: "command_prototype",
    history_event_message_name: "Command Prototype"
}

/// A CommandProtoype joins the [`Funcs`](crate::Func) which are used in
/// (`Workflows`)(crate::Workflow) to the context in which they are executed (schema variant or
/// component context)
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CommandPrototype {
    pk: CommandPrototypePk,
    id: CommandPrototypeId,
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

impl CommandPrototype {
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        func_id: FuncId,
        CommandPrototypeContext {
            component_id,
            schema_variant_id,
        }: CommandPrototypeContext,
    ) -> CommandPrototypeResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM command_prototype_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &component_id,
                    &schema_variant_id,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(func_id, Pk(FuncId), CommandPrototypeResult);
    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        CommandPrototypeResult
    );
    standard_model_accessor!(component_id, Pk(ComponentId), CommandPrototypeResult);

    pub async fn find_for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> CommandPrototypeResult<Vec<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query(FIND_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), &func_id])
            .await?;

        Ok(standard_model::objects_from_rows(row)?)
    }

    pub async fn find_for_func_and_schema_variant(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> CommandPrototypeResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                FIND_FOR_FUNC_AND_SCHEMA_VARIANT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &schema_variant_id,
                ],
            )
            .await?;

        Ok(standard_model::object_option_from_row_option(row)?)
    }

    pub async fn find_for_context(
        ctx: &DalContext,
        CommandPrototypeContext {
            component_id,
            schema_variant_id,
        }: CommandPrototypeContext,
    ) -> CommandPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &schema_variant_id,
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }
}

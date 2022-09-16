use crate::DalContext;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data::{NatsError, PgError};
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::FuncId, impl_standard_model, pk, standard_model, standard_model_accessor, CodeLanguage,
    ComponentId, HistoryEventError, SchemaId, SchemaVariantId, StandardModel, StandardModelError,
    SystemId, Timestamp, Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum CodeGenerationPrototypeError {
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
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("component error: {0}")]
    Component(String),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
}

pub type CodeGenerationPrototypeResult<T> = Result<T, CodeGenerationPrototypeError>;

pub const UNSET_ID_VALUE: i64 = -1;
const FIND_FOR_CONTEXT: &str =
    include_str!("./queries/code_generation_prototype_find_for_context.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CodeGenerationPrototypeContext {
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for CodeGenerationPrototypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerationPrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: UNSET_ID_VALUE.into(),
            schema_id: UNSET_ID_VALUE.into(),
            schema_variant_id: UNSET_ID_VALUE.into(),
            system_id: UNSET_ID_VALUE.into(),
        }
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn set_component_id(&mut self, component_id: ComponentId) {
        self.component_id = component_id;
    }

    pub fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub fn set_schema_id(&mut self, schema_id: SchemaId) {
        self.schema_id = schema_id;
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) {
        self.schema_variant_id = schema_variant_id;
    }

    pub fn system_id(&self) -> SystemId {
        self.system_id
    }

    pub fn set_system_id(&mut self, system_id: SystemId) {
        self.system_id = system_id;
    }
}

pk!(CodeGenerationPrototypePk);
pk!(CodeGenerationPrototypeId);

// An CodeGenerationPrototype joins a `Func` to the context in which
// the component that is created with it can use to generate a CodeGenerationResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CodeGenerationPrototype {
    pk: CodeGenerationPrototypePk,
    id: CodeGenerationPrototypeId,
    func_id: FuncId,
    args: serde_json::Value,
    format: CodeLanguage,
    #[serde(flatten)]
    context: CodeGenerationPrototypeContext,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: CodeGenerationPrototype,
    pk: CodeGenerationPrototypePk,
    id: CodeGenerationPrototypeId,
    table_name: "code_generation_prototypes",
    history_event_label_base: "code_generation_prototype",
    history_event_message_name: "CodeGeneration Prototype"
}

impl CodeGenerationPrototype {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        func_id: FuncId,
        args: serde_json::Value,
        format: CodeLanguage,
        context: CodeGenerationPrototypeContext,
    ) -> CodeGenerationPrototypeResult<Self> {
        let row = ctx.txns().pg().query_one(
                "SELECT object FROM code_generation_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[ctx.write_tenancy(), ctx.visibility(),
                    &func_id,
                    &args,
                    &format.as_ref(),
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(func_id, Pk(FuncId), CodeGenerationPrototypeResult);
    standard_model_accessor!(args, Json<JsonValue>, CodeGenerationPrototypeResult);
    standard_model_accessor!(format, Enum(CodeLanguage), CodeGenerationPrototypeResult);

    #[allow(clippy::too_many_arguments)]
    pub async fn find_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        system_id: SystemId,
    ) -> CodeGenerationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                FIND_FOR_CONTEXT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &system_id,
                    &schema_variant_id,
                    &schema_id,
                ],
            )
            .await?;
        let object = standard_model::objects_from_rows(rows)?;
        Ok(object)
    }
}

#[cfg(test)]
mod test {
    use super::CodeGenerationPrototypeContext;

    #[test]
    fn context_builder() {
        let mut c = CodeGenerationPrototypeContext::new();
        c.set_component_id(22.into());
        assert_eq!(c.component_id(), 22.into());
    }
}

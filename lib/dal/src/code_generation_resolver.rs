use crate::DalContext;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::{binding::FuncBindingId, FuncId},
    impl_standard_model, pk, standard_model, standard_model_accessor, CodeGenerationPrototypeId,
    ComponentId, HistoryEventError, SchemaId, SchemaVariantId, StandardModel, StandardModelError,
    SystemId, Timestamp, Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum CodeGenerationResolverError {
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

pub type CodeGenerationResolverResult<T> = Result<T, CodeGenerationResolverError>;

pub const UNSET_ID_VALUE: i64 = -1;
const FIND_FOR_PROTOTYPE: &str =
    include_str!("./queries/code_generation_resolver_find_for_context.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CodeGenerationResolverContext {
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for CodeGenerationResolverContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerationResolverContext {
    pub fn new() -> Self {
        CodeGenerationResolverContext {
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

pk!(CodeGenerationResolverPk);
pk!(CodeGenerationResolverId);

// An CodeGenerationResolver joins a `FuncBinding` to the context in which
// its corresponding `FuncBindingResultValue` is consumed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CodeGenerationResolver {
    pk: CodeGenerationResolverPk,
    id: CodeGenerationResolverId,
    code_generation_prototype_id: CodeGenerationPrototypeId,
    func_id: FuncId,
    func_binding_id: FuncBindingId,
    #[serde(flatten)]
    context: CodeGenerationResolverContext,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: CodeGenerationResolver,
    pk: CodeGenerationResolverPk,
    id: CodeGenerationResolverId,
    table_name: "code_generation_resolvers",
    history_event_label_base: "code_generation_resolver",
    history_event_message_name: "CodeGeneration Resolver"
}

impl CodeGenerationResolver {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        code_generation_prototype_id: CodeGenerationPrototypeId,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        context: CodeGenerationResolverContext,
    ) -> CodeGenerationResolverResult<Self> {
        let row = ctx.txns().pg().query_one(
                "SELECT object FROM code_generation_resolver_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[ctx.write_tenancy(), ctx.visibility(),
                    &code_generation_prototype_id,
                    &func_id,
                    &func_binding_id,
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

    standard_model_accessor!(
        code_generation_prototype_id,
        Pk(CodeGenerationPrototypeId),
        CodeGenerationResolverResult
    );
    standard_model_accessor!(func_id, Pk(FuncId), CodeGenerationResolverResult);
    standard_model_accessor!(
        func_binding_id,
        Pk(FuncBindingId),
        CodeGenerationResolverResult
    );

    pub async fn find_for_prototype_and_component(
        ctx: &DalContext<'_, '_>,
        code_generation_prototype_id: &CodeGenerationPrototypeId,
        component_id: &ComponentId,
    ) -> CodeGenerationResolverResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                FIND_FOR_PROTOTYPE,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    code_generation_prototype_id,
                    component_id,
                ],
            )
            .await?;
        let object = standard_model::objects_from_rows(rows)?;
        Ok(object)
    }
}

#[cfg(test)]
mod test {
    use super::CodeGenerationResolverContext;

    #[test]
    fn context_builder() {
        let mut c = CodeGenerationResolverContext::new();
        c.set_component_id(15.into());
        assert_eq!(c.component_id(), 15.into());
    }
}

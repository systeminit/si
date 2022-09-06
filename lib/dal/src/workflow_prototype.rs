use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data::{NatsError, PgError};
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::FuncId,
    impl_standard_model, pk,
    standard_model::{self, objects_from_rows},
    standard_model_accessor,
    workflow_resolver::WorkflowResolverContext,
    ComponentId, DalContext, Func, FuncBinding, FuncBindingError, HistoryEventError, SchemaId,
    SchemaVariantId, StandardModel, StandardModelError, SystemId, Timestamp, Visibility,
    WorkflowError, WorkflowResolver, WorkflowResolverError, WorkflowView, WriteTenancy, WsEvent,
    WsEventError,
};

#[derive(Error, Debug)]
pub enum WorkflowPrototypeError {
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
    #[error(transparent)]
    Workflow(#[from] WorkflowError),
    #[error(transparent)]
    WorkflowResolver(#[from] WorkflowResolverError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
    #[error("component error: {0}")]
    Component(String),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("func not found {0}")]
    FuncNotFound(FuncId),
}

pub type WorkflowPrototypeResult<T> = Result<T, WorkflowPrototypeError>;

const FIND_FOR_CONTEXT: &str = include_str!("./queries/workflow_prototype_find_for_context.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowPrototypeContext {
    pub component_id: ComponentId,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub system_id: SystemId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for WorkflowPrototypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowPrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
            system_id: SystemId::NONE,
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

pk!(WorkflowPrototypePk);
pk!(WorkflowPrototypeId);

// An WorkflowPrototype joins a `Func` to the context in which
// the component that is created with it can use to generate a WorkflowResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowPrototype {
    pk: WorkflowPrototypePk,
    id: WorkflowPrototypeId,
    func_id: FuncId,
    args: serde_json::Value,
    title: String,
    description: Option<String>,
    link: Option<String>,
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: WorkflowPrototype,
    pk: WorkflowPrototypePk,
    id: WorkflowPrototypeId,
    table_name: "workflow_prototypes",
    history_event_label_base: "workflow_prototype",
    history_event_message_name: "Workflow Prototype"
}

impl WorkflowPrototype {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_, '_>,
        func_id: FuncId,
        args: serde_json::Value,
        context: WorkflowPrototypeContext,
        title: impl Into<String>,
    ) -> WorkflowPrototypeResult<Self> {
        let title = title.into();
        let row = ctx.txns().pg().query_one(
                "SELECT object FROM workflow_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[ctx.write_tenancy(), ctx.visibility(),
                    &func_id,
                    &args,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                    &title,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(func_id, Pk(FuncId), WorkflowPrototypeResult);
    standard_model_accessor!(args, Json<JsonValue>, WorkflowPrototypeResult);
    standard_model_accessor!(title, String, WorkflowPrototypeResult);
    standard_model_accessor!(description, Option<String>, WorkflowPrototypeResult);
    standard_model_accessor!(link, Option<String>, WorkflowPrototypeResult);
    standard_model_accessor!(schema_id, Pk(SchemaId), WorkflowPrototypeResult);
    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        WorkflowPrototypeResult
    );
    standard_model_accessor!(component_id, Pk(ComponentId), WorkflowPrototypeResult);

    standard_model_accessor!(system_id, Pk(SystemId), WorkflowPrototypeResult);

    pub async fn resolve(
        &self,
        ctx: &DalContext<'_, '_, '_>,
    ) -> WorkflowPrototypeResult<WorkflowResolver> {
        let mut context = WorkflowResolverContext::new();
        context.set_component_id(self.component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);
        context.set_system_id(self.system_id);
        match WorkflowResolver::find_for_prototype(ctx, self.id(), context.clone())
            .await?
            .pop()
        {
            Some(resolver) => Ok(resolver),
            None => {
                let identity = Func::find_by_attr(ctx, "name", &"si:identity")
                    .await?
                    .pop()
                    .ok_or_else(|| WorkflowError::MissingWorkflow("si:identity".to_owned()))?;
                let (func_binding, _) = FuncBinding::find_or_create_and_execute(
                    ctx,
                    serde_json::json!({ "identity": null }),
                    *identity.id(),
                )
                .await?;
                let mut resolver = WorkflowResolver::new(
                    ctx,
                    self.id,
                    *identity.id(),
                    *func_binding.id(),
                    context.clone(),
                )
                .await?;

                let func = Func::get_by_id(ctx, &self.func_id())
                    .await?
                    .ok_or_else(|| WorkflowPrototypeError::FuncNotFound(self.func_id()))?;
                let tree = WorkflowView::resolve(ctx, &func).await?;

                let args = serde_json::json!({ "identity": serde_json::to_value(tree)? });
                let (func_binding, _) =
                    FuncBinding::find_or_create_and_execute(ctx, args, *identity.id()).await?;
                resolver
                    .set_func_binding_id(ctx, *func_binding.id())
                    .await?;

                WsEvent::change_set_written(ctx).publish(ctx).await?;

                Ok(resolver)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn find_for_context(
        ctx: &DalContext<'_, '_, '_>,
        component_id: ComponentId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        system_id: SystemId,
    ) -> WorkflowPrototypeResult<Vec<Self>> {
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
        let object = objects_from_rows(rows)?;
        Ok(object)
    }

    pub fn context(&self) -> WorkflowPrototypeContext {
        let mut context = WorkflowPrototypeContext::new();
        context.set_component_id(self.component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);
        context.set_system_id(self.system_id);

        context
    }
}

#[cfg(test)]
mod test {
    use super::WorkflowPrototypeContext;

    #[test]
    fn context_builder() {
        let mut c = WorkflowPrototypeContext::new();
        c.set_component_id(22.into());
        assert_eq!(c.component_id(), 22.into());
    }
}

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::FuncId, impl_standard_model, pk, standard_model, standard_model_accessor,
    workflow_resolver::WorkflowResolverContext, Component, ComponentId, ComponentView, DalContext,
    Func, FuncBinding, FuncBindingError, FuncError, HistoryEventError, SchemaId, SchemaVariantId,
    StandardModel, StandardModelError, Tenancy, Timestamp, TransactionsError, Visibility,
    WorkflowError, WorkflowResolver, WorkflowResolverError, WorkflowView, WsEvent, WsEventError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkflowPrototypeError {
    #[error("component error: {0}")]
    Component(String),
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error("func not found {0}")]
    FuncNotFound(FuncId),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("schema doesnt match, prototype = {0}, component = {1}")]
    SchemaDoesntMatch(WorkflowPrototypeId, ComponentId),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    Workflow(#[from] WorkflowError),
    #[error(transparent)]
    WorkflowResolver(#[from] WorkflowResolverError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type WorkflowPrototypeResult<T> = Result<T, WorkflowPrototypeError>;

const FIND_FOR_CONTEXT: &str = include_str!("./queries/workflow_prototype_find_for_context.sql");
const FIND_FOR_FUNC: &str = include_str!("./queries/workflow_prototype_find_for_func.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowPrototypeContext {
    pub component_id: ComponentId,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
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
    #[serde(flatten)]
    tenancy: Tenancy,
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
        ctx: &DalContext,
        func_id: FuncId,
        args: serde_json::Value,
        context: WorkflowPrototypeContext,
        title: impl Into<String>,
    ) -> WorkflowPrototypeResult<Self> {
        let title = title.into();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM workflow_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &args,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
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

    /// For the given [`WorkflowPrototype`](Self), find or create a
    /// [`WorkflowResolver`](crate::workflow_resolver::WorkflowResolver) corresponding to the
    /// identity [`Func`](crate::Func).
    ///
    /// Found resolvers are not modified.
    pub async fn resolve(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WorkflowPrototypeResult<WorkflowResolver> {
        let component = if component_id.is_some() {
            let deleted_ctx = &ctx.clone_with_delete_visibility();
            let component = Component::get_by_id(deleted_ctx, &component_id)
                .await?
                .ok_or(WorkflowPrototypeError::ComponentNotFound(component_id))?;
            let schema = component
                .schema(ctx)
                .await
                .map_err(|err| WorkflowPrototypeError::Component(err.to_string()))?
                .ok_or(WorkflowPrototypeError::SchemaNotFound)?;
            let schema_variant = component
                .schema_variant(ctx)
                .await
                .map_err(|err| WorkflowPrototypeError::Component(err.to_string()))?
                .ok_or(WorkflowPrototypeError::SchemaVariantNotFound)?;
            if *schema.id() != self.schema_id || *schema_variant.id() != self.schema_variant_id {
                return Err(WorkflowPrototypeError::SchemaDoesntMatch(
                    self.id,
                    component_id,
                ));
            }

            let component_view = ComponentView::new(ctx, *component.id())
                .await
                .map_err(|err| WorkflowPrototypeError::Component(err.to_string()))?;
            Some(component_view)
        } else {
            None
        };

        let mut context = WorkflowResolverContext::new();
        context.set_component_id(self.component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);
        let resolver = WorkflowResolver::find_for_prototype(ctx, self.id(), context.clone())
            .await?
            .pop();

        let identity_func = Func::identity_func(ctx).await?;

        let mut resolver = if let Some(resolver) = resolver {
            resolver
        } else {
            let (identity_func_binding, _) = FuncBinding::create_and_execute(
                ctx,
                serde_json::json!({ "identity": null }),
                *identity_func.id(),
            )
            .await?;
            WorkflowResolver::new(
                ctx,
                self.id,
                *identity_func.id(),
                *identity_func_binding.id(),
                context.clone(),
            )
            .await?
        };

        // FIXME(nick,wendy): why create the resolver before getting the workflow tree?
        // It seems like we can assemble the args first? However, there might be a scenario
        // where the workflow tree assembly requires the existence of the resolver first?
        let workflow_prototype_func = Func::get_by_id(ctx, &self.func_id())
            .await?
            .ok_or_else(|| WorkflowPrototypeError::FuncNotFound(self.func_id()))?;
        let tree = WorkflowView::resolve(
            ctx,
            &workflow_prototype_func,
            serde_json::to_value(component)?,
        )
        .await?;
        let args = serde_json::json!({ "identity": serde_json::to_value(tree)? });

        // Serialize the tree into the arguments for the identity function.
        let (func_binding, _) =
            FuncBinding::create_and_execute(ctx, args, *identity_func.id()).await?;
        resolver
            .set_func_binding_id(ctx, *func_binding.id())
            .await?;

        WsEvent::change_set_written(ctx)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(resolver)
    }

    pub async fn find_for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> WorkflowPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(FIND_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), &func_id])
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn find_for_context(
        ctx: &DalContext,
        context: WorkflowPrototypeContext,
    ) -> WorkflowPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context.component_id,
                    &context.schema_variant_id,
                    &context.schema_id,
                ],
            )
            .await?;
        let object = standard_model::objects_from_rows(rows)?;
        Ok(object)
    }

    pub fn context(&self) -> WorkflowPrototypeContext {
        let mut context = WorkflowPrototypeContext::new();
        context.set_component_id(self.component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);

        context
    }
}

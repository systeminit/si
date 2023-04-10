use crate::{DalContext, TransactionsError};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError},
    func::{binding::FuncBindingId, FuncId},
    impl_standard_model, pk,
    standard_model::{self, objects_from_rows},
    standard_model_accessor, ComponentId, FuncBinding, HistoryEventError, SchemaId,
    SchemaVariantId, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
    WorkflowPrototypeId, WorkflowTree,
};

#[derive(Error, Debug)]
pub enum WorkflowResolverError {
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
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
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workflow not resolved yet")]
    NotResolved(WorkflowResolverId),
    #[error("func binding not found {0}")]
    FuncBindingNotFound(FuncBindingId),
}

pub type WorkflowResolverResult<T> = Result<T, WorkflowResolverError>;

const FIND_FOR_PROTOTYPE: &str = include_str!("./queries/workflow_resolver_find_for_prototype.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowResolverContext {
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for WorkflowResolverContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowResolverContext {
    pub fn new() -> Self {
        WorkflowResolverContext {
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

pk!(WorkflowResolverPk);
pk!(WorkflowResolverId);

/// A [`WorkflowResolver`] joins a [`FuncBinding`](crate::FuncBinding) to the
/// [`WorkflowResolverContext`] in which its corresponding
/// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) is consumed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowResolver {
    pk: WorkflowResolverPk,
    id: WorkflowResolverId,
    workflow_prototype_id: WorkflowPrototypeId,
    func_id: FuncId,
    func_binding_id: FuncBindingId,
    #[serde(flatten)]
    context: WorkflowResolverContext,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: WorkflowResolver,
    pk: WorkflowResolverPk,
    id: WorkflowResolverId,
    table_name: "workflow_resolvers",
    history_event_label_base: "workflow_resolver",
    history_event_message_name: "Workflow Resolver"
}

impl WorkflowResolver {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        workflow_prototype_id: WorkflowPrototypeId,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        context: WorkflowResolverContext,
    ) -> WorkflowResolverResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM workflow_resolver_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &workflow_prototype_id,
                    &func_id,
                    &func_binding_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    pub async fn tree(&self, ctx: &DalContext) -> WorkflowResolverResult<WorkflowTree> {
        let func_binding = FuncBinding::get_by_id(ctx, &self.func_binding_id())
            .await?
            .ok_or_else(|| WorkflowResolverError::FuncBindingNotFound(self.func_binding_id()))?;
        let value = FuncBindingReturnValue::get_by_func_binding_id(ctx, *func_binding.id()).await?;
        let value = value.as_ref().and_then(|v| v.value());
        let tree = WorkflowTree::deserialize(
            value.ok_or_else(|| WorkflowResolverError::NotResolved(*self.id()))?,
        )?;
        Ok(tree)
    }

    standard_model_accessor!(
        workflow_prototype_id,
        Pk(WorkflowPrototypeId),
        WorkflowResolverResult
    );
    standard_model_accessor!(func_id, Pk(FuncId), WorkflowResolverResult);
    standard_model_accessor!(func_binding_id, Pk(FuncBindingId), WorkflowResolverResult);

    pub async fn find_for_prototype(
        ctx: &DalContext,
        workflow_prototype_id: &WorkflowPrototypeId,
        context: WorkflowResolverContext,
    ) -> WorkflowResolverResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_PROTOTYPE,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    workflow_prototype_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = objects_from_rows(rows)?;
        Ok(object)
    }
}

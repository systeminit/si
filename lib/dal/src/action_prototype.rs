use std::default::Default;

use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display};
use thiserror::Error;

use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, ComponentId, DalContext,
    HistoryEventError, SchemaId, SchemaVariantId, StandardModel, StandardModelError, Timestamp,
    Visibility, WorkflowPrototype, WorkflowPrototypeId, WriteTenancy,
};

const FIND_BY_NAME: &str = include_str!("./queries/action_prototype_find_by_name.sql");

#[derive(Error, Debug)]
pub enum ActionPrototypeError {
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
    #[error("not found with name {0}")]
    NotFoundByName(String),
    #[error("workflow prototype {0} not found")]
    WorkflowPrototypeNotFound(WorkflowPrototypeId),
}

pub type ActionPrototypeResult<T> = Result<T, ActionPrototypeError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ActionPrototypeContext {
    pub component_id: ComponentId,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
}

// Describes how an action affects the world
#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ActionKind {
    // Create a new resource
    Create,
    // Internal only action or action with multiple effects
    Other,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for ActionPrototypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionPrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
        }
    }

    pub fn new_for_context_field(context_field: ActionPrototypeContextField) -> Self {
        match context_field {
            ActionPrototypeContextField::Schema(schema_id) => ActionPrototypeContext {
                component_id: ComponentId::NONE,
                schema_id,
                schema_variant_id: SchemaVariantId::NONE,
            },
            ActionPrototypeContextField::SchemaVariant(schema_variant_id) => {
                ActionPrototypeContext {
                    component_id: ComponentId::NONE,
                    schema_id: SchemaId::NONE,
                    schema_variant_id,
                }
            }
            ActionPrototypeContextField::Component(component_id) => ActionPrototypeContext {
                component_id,
                schema_id: SchemaId::NONE,
                schema_variant_id: SchemaVariantId::NONE,
            },
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

pk!(ActionPrototypePk);
pk!(ActionPrototypeId);

// An ActionPrototype joins a `WorkflowPrototype` to the context in which
// the component that is created with it can use to generate a ConfirmationResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ActionPrototype {
    pk: ActionPrototypePk,
    id: ActionPrototypeId,
    workflow_prototype_id: WorkflowPrototypeId,
    name: String,
    kind: ActionKind,
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ActionPrototypeContextField {
    Component(ComponentId),
    Schema(SchemaId),
    SchemaVariant(SchemaVariantId),
}

impl From<ComponentId> for ActionPrototypeContextField {
    fn from(component_id: ComponentId) -> Self {
        ActionPrototypeContextField::Component(component_id)
    }
}

impl From<SchemaId> for ActionPrototypeContextField {
    fn from(schema_id: SchemaId) -> Self {
        ActionPrototypeContextField::Schema(schema_id)
    }
}

impl From<SchemaVariantId> for ActionPrototypeContextField {
    fn from(schema_variant_id: SchemaVariantId) -> Self {
        ActionPrototypeContextField::SchemaVariant(schema_variant_id)
    }
}

impl_standard_model! {
    model: ActionPrototype,
    pk: ActionPrototypePk,
    id: ActionPrototypeId,
    table_name: "action_prototypes",
    history_event_label_base: "action_prototype",
    history_event_message_name: "Action Prototype"
}

impl ActionPrototype {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        workflow_prototype_id: WorkflowPrototypeId,
        name: &str,
        kind: ActionKind,
        context: ActionPrototypeContext,
    ) -> ActionPrototypeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM action_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &workflow_prototype_id,
                    &name,
                    &kind.as_ref(),
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn find_by_name(
        ctx: &DalContext,
        name: &str,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) -> ActionPrototypeResult<Option<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query_opt(
                FIND_BY_NAME,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &name,
                    &schema_variant_id,
                    &schema_id,
                ],
            )
            .await?;
        let object = standard_model::option_object_from_row(rows)?;
        Ok(object)
    }

    pub async fn workflow_prototype(
        &self,
        ctx: &DalContext,
    ) -> ActionPrototypeResult<WorkflowPrototype> {
        WorkflowPrototype::get_by_id(ctx, &self.workflow_prototype_id)
            .await?
            .ok_or(ActionPrototypeError::WorkflowPrototypeNotFound(
                self.workflow_prototype_id,
            ))
    }

    standard_model_accessor!(
        workflow_prototype_id,
        Pk(WorkflowPrototypeId),
        ActionPrototypeResult
    );
    standard_model_accessor!(schema_id, Pk(SchemaId), ActionPrototypeResult);
    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        ActionPrototypeResult
    );
    standard_model_accessor!(component_id, Pk(ComponentId), ActionPrototypeResult);

    standard_model_accessor!(name, String, ActionPrototypeResult);
    standard_model_accessor!(kind, Enum(ActionKind), ActionPrototypeResult);

    pub fn context(&self) -> ActionPrototypeContext {
        let mut context = ActionPrototypeContext::new();
        context.set_component_id(self.component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);

        context
    }
}

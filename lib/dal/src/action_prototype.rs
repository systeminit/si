use std::default::Default;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};
use thiserror::Error;

use si_data_nats::NatsError;
use si_data_pg::PgError;
use si_pkg::ActionFuncSpecKind;
use telemetry::prelude::*;

use crate::{
    component::view::ComponentViewError, func::backend::js_action::ActionRunResult,
    impl_standard_model, pk, standard_model, standard_model_accessor, Component, ComponentId,
    ComponentView, DalContext, FuncBinding, FuncBindingError, FuncBindingReturnValueError, FuncId,
    HistoryEventError, SchemaVariantId, StandardModel, StandardModelError, Tenancy, Timestamp,
    TransactionsError, Visibility, WsEvent, WsEventError,
};

const FIND_FOR_CONTEXT: &str = include_str!("./queries/action_prototype/find_for_context.sql");
const FIND_FOR_CONTEXT_AND_KIND: &str =
    include_str!("./queries/action_prototype/find_for_context_and_kind.sql");
const FIND_FOR_FUNC: &str = include_str!("./queries/action_prototype/find_for_func.sql");
const FIND_FOR_CONTEXT_AND_FUNC: &str =
    include_str!("./queries/action_prototype/find_for_context_and_func.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionPrototypeView {
    id: ActionPrototypeId,
    name: String,
}

impl ActionPrototypeView {
    pub fn new(prototype: ActionPrototype) -> Self {
        Self {
            id: prototype.id,
            name: prototype.name().map_or_else(
                || match prototype.kind() {
                    ActionKind::Create => "create".to_owned(),
                    ActionKind::Delete => "delete".to_owned(),
                    ActionKind::Other => "other".to_owned(),
                    ActionKind::Refresh => " refresh".to_owned(),
                },
                ToOwned::to_owned,
            ),
        }
    }
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ActionPrototypeError {
    #[error("component error: {0}")]
    Component(String),
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error(transparent)]
    ComponentView(#[from] ComponentViewError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("action Func {0} not found for ActionPrototype {1}")]
    FuncNotFound(FuncId, ActionPrototypeId),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("not found with kind {0} for context {1:?}")]
    NotFoundByKindAndContext(ActionKind, ActionPrototypeContext),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
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
    WsEvent(#[from] WsEventError),
}

pub type ActionPrototypeResult<T> = Result<T, ActionPrototypeError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct ActionPrototypeContext {
    pub schema_variant_id: SchemaVariantId,
}

/// Describes how an [`Action`](ActionPrototype) affects the world.
#[remain::sorted]
#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy, Hash)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ActionKind {
    /// The [`action`](ActionPrototype) creates a new "resource".
    Create,
    /// The [`action`](ActionPrototype) deletes an existing "resource".
    Delete,
    /// The [`action`](ActionPrototype) is "internal only" or has multiple effects.
    Other,
    /// The [`action`](ActionPrototype) that refreshes an existing "resource".
    Refresh,
}

impl From<ActionFuncSpecKind> for ActionKind {
    fn from(value: ActionFuncSpecKind) -> Self {
        match value {
            ActionFuncSpecKind::Create => ActionKind::Create,
            ActionFuncSpecKind::Refresh => ActionKind::Refresh,
            ActionFuncSpecKind::Other => ActionKind::Other,
            ActionFuncSpecKind::Delete => ActionKind::Delete,
        }
    }
}

impl From<&ActionKind> for ActionFuncSpecKind {
    fn from(value: &ActionKind) -> Self {
        match value {
            ActionKind::Create => ActionFuncSpecKind::Create,
            ActionKind::Refresh => ActionFuncSpecKind::Refresh,
            ActionKind::Other => ActionFuncSpecKind::Other,
            ActionKind::Delete => ActionFuncSpecKind::Delete,
        }
    }
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
            schema_variant_id: SchemaVariantId::NONE,
        }
    }

    pub fn new_for_context_field(context_field: ActionPrototypeContextField) -> Self {
        match context_field {
            ActionPrototypeContextField::SchemaVariant(schema_variant_id) => {
                ActionPrototypeContext { schema_variant_id }
            }
        }
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
    func_id: FuncId,
    kind: ActionKind,
    name: Option<String>,
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
pub enum ActionPrototypeContextField {
    SchemaVariant(SchemaVariantId),
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
        func_id: FuncId,
        kind: ActionKind,
        context: ActionPrototypeContext,
    ) -> ActionPrototypeResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM action_prototype_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &kind.as_ref(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    pub async fn find_for_context(
        ctx: &DalContext,
        context: ActionPrototypeContext,
    ) -> ActionPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn find_for_context_and_kind(
        ctx: &DalContext,
        kind: ActionKind,
        context: ActionPrototypeContext,
    ) -> ActionPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_CONTEXT_AND_KIND,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &kind.as_ref(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn find_for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> ActionPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(FIND_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), &func_id])
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn find_for_context_and_func(
        ctx: &DalContext,
        context: ActionPrototypeContext,
        func_id: FuncId,
    ) -> ActionPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_CONTEXT_AND_FUNC,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context.schema_variant_id(),
                    &func_id,
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        ActionPrototypeResult
    );
    standard_model_accessor!(name, Option<String>, ActionPrototypeResult);
    standard_model_accessor!(func_id, Pk(FuncId), ActionPrototypeResult);
    standard_model_accessor!(kind, Enum(ActionKind), ActionPrototypeResult);

    pub fn context(&self) -> ActionPrototypeContext {
        let mut context = ActionPrototypeContext::new();
        context.set_schema_variant_id(self.schema_variant_id);

        context
    }

    pub async fn run(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
        trigger_dependent_values_update: bool,
    ) -> ActionPrototypeResult<Option<ActionRunResult>> {
        let component_view = ComponentView::new(ctx, component_id).await?;
        let (_, return_value) = FuncBinding::create_and_execute(
            ctx,
            serde_json::to_value(component_view)?,
            self.func_id(),
        )
        .await?;

        let mut logs = vec![];
        for stream_part in return_value
            .get_output_stream(ctx)
            .await?
            .unwrap_or_default()
        {
            logs.push(stream_part);
        }

        logs.sort_by_key(|log| log.timestamp);

        Ok(match return_value.value() {
            Some(value) => {
                let mut run_result: ActionRunResult = serde_json::from_value(value.clone())?;
                run_result.logs = logs.iter().map(|l| l.message.clone()).collect();

                let deleted_ctx = &ctx.clone_with_delete_visibility();
                let mut component = Component::get_by_id(deleted_ctx, &component_id)
                    .await?
                    .ok_or(ActionPrototypeError::ComponentNotFound(component_id))?;

                if component.needs_destroy() && run_result.payload.is_none() {
                    component
                        .set_needs_destroy(deleted_ctx, false)
                        .await
                        .map_err(|e| ActionPrototypeError::Component(e.to_string()))?;
                }

                if component
                    .set_resource(ctx, run_result.clone(), trigger_dependent_values_update)
                    .await
                    .map_err(|e| ActionPrototypeError::Component(e.to_string()))?
                {
                    WsEvent::resource_refreshed(ctx, *component.id())
                        .await?
                        .publish_on_commit(ctx)
                        .await?;
                }

                Some(run_result)
            }
            None => None,
        })
    }
}

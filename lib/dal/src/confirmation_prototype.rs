use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data_pg::PgError;
use telemetry::prelude::*;

use crate::{
    func::backend::js_confirmation::{ConfirmationResult, FuncBackendJsConfirmationArgs},
    func::FuncId,
    impl_prototype_list_for_func, impl_standard_model, pk,
    prototype_context::{HasPrototypeContext, PrototypeContext},
    standard_model, standard_model_accessor, ActionPrototype, ActionPrototypeError, Component,
    ComponentError, ComponentId, ConfirmationResolver, ConfirmationResolverContext,
    ConfirmationResolverError, DalContext, Func, FuncBinding, FuncBindingError, FuncBindingId,
    FuncError, HistoryEventError, SchemaId, SchemaVariantId, StandardModel, StandardModelError,
    Timestamp, Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum ConfirmationPrototypeError {
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error(transparent)]
    ConfirmationResolver(#[from] ConfirmationResolverError),
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error(transparent)]
    Func(#[from] FuncError),

    #[error("not found by id: {0}")]
    NotFound(ConfirmationPrototypeId),
}

pub type ConfirmationPrototypeResult<T> = Result<T, ConfirmationPrototypeError>;

const LIST_FOR_CONTEXT: &str = include_str!("queries/confirmation_prototype_list_for_context.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationPrototypeContext {
    pub component_id: ComponentId,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for ConfirmationPrototypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PrototypeContext for ConfirmationPrototypeContext {
    fn component_id(&self) -> ComponentId {
        self.component_id
    }

    fn set_component_id(&mut self, component_id: ComponentId) {
        self.component_id = component_id;
    }

    fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    fn set_schema_id(&mut self, schema_id: SchemaId) {
        self.schema_id = schema_id;
    }
    fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) {
        self.schema_variant_id = schema_variant_id;
    }
}

impl ConfirmationPrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
        }
    }
}

pk!(ConfirmationPrototypePk);
pk!(ConfirmationPrototypeId);

// An ConfirmationPrototype joins a `Func` to the context in which
// the component that is created with it can use to generate a ConfirmationResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationPrototype {
    pk: ConfirmationPrototypePk,
    id: ConfirmationPrototypeId,
    name: String,
    success_description: Option<String>,
    failure_description: Option<String>,
    provider: Option<String>,
    func_id: FuncId,
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

impl_standard_model! {
    model: ConfirmationPrototype,
    pk: ConfirmationPrototypePk,
    id: ConfirmationPrototypeId,
    table_name: "confirmation_prototypes",
    history_event_label_base: "confirmation_prototype",
    history_event_message_name: "Confirmation Prototype"
}

impl HasPrototypeContext<ConfirmationPrototypeContext> for ConfirmationPrototype {
    fn context(&self) -> ConfirmationPrototypeContext {
        let mut context = ConfirmationPrototypeContext::new();
        context.set_component_id(self.component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);

        context
    }

    fn new_context() -> ConfirmationPrototypeContext {
        ConfirmationPrototypeContext::new()
    }
}

impl ConfirmationPrototype {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: &str,
        func_id: FuncId,
        context: ConfirmationPrototypeContext,
    ) -> ConfirmationPrototypeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM confirmation_prototype_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &func_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    pub async fn prepare(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ConfirmationPrototypeResult<ConfirmationResolver> {
        let mut context = ConfirmationResolverContext::new();
        context.set_component_id(component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);
        if let Some(mut resolver) =
            ConfirmationResolver::find_for_prototype(ctx, self.id(), context.clone()).await?
        {
            resolver
                .set_func_binding_id(ctx, FuncBindingId::NONE)
                .await?;
            resolver.set_success(ctx, None::<bool>).await?;
            resolver.set_message(ctx, None::<&str>).await?;
            resolver.remove_all_recommended_actions(ctx).await?;
            Ok(resolver)
        } else {
            Ok(ConfirmationResolver::new(
                ctx,
                *self.id(),
                None,
                None,
                Vec::new(),
                self.func_id(),
                FuncBindingId::NONE,
                context,
            )
            .await?)
        }
    }

    pub async fn run(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ConfirmationPrototypeResult<ConfirmationResolver> {
        let args = FuncBackendJsConfirmationArgs {
            component: Component::view(ctx, component_id).await?.into(),
        };

        let json_args = serde_json::to_value(args)?;
        let func = Func::get_by_id(ctx, &self.func_id)
            .await?
            .ok_or(FuncError::NotFound(self.func_id))?;
        let (func_binding, _) =
            FuncBinding::find_or_create(ctx, json_args, self.func_id(), *func.backend_kind())
                .await?;
        let func_binding_return_value = func_binding.execute(ctx).await?;

        let (success, message, recommended_actions) = if let Some(mut value) =
            func_binding_return_value
                .value()
                .map(ConfirmationResult::deserialize)
                .transpose()?
        {
            let mut recommended_actions = Vec::with_capacity(value.recommended_actions.len());
            for action_name in value.recommended_actions {
                let action = ActionPrototype::find_by_name(
                    ctx,
                    &action_name,
                    self.schema_id,
                    self.schema_variant_id,
                )
                .await?;
                if let Some(action) = action {
                    recommended_actions.push(action);
                } else {
                    value.success = false;
                    value.message = Some(format!("Unable to find action {}", action_name));
                    recommended_actions.clear();
                    break;
                };
            }
            (value.success, value.message, recommended_actions)
        } else {
            (
                false,
                Some(format!(
                    "Unable to deserialize func_binding_return_value's value: {:?}",
                    func_binding_return_value.value()
                )),
                Vec::new(),
            )
        };

        let mut context = ConfirmationResolverContext::new();
        context.set_component_id(component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);
        if let Some(mut resolver) =
            ConfirmationResolver::find_for_prototype(ctx, self.id(), context.clone()).await?
        {
            resolver.set_success(ctx, Some(success)).await?;
            resolver.set_message(ctx, message).await?;
            resolver.remove_all_recommended_actions(ctx).await?;

            for recommended_action in recommended_actions {
                resolver
                    .add_recommended_action(ctx, recommended_action.id())
                    .await?;
            }
            Ok(resolver)
        } else {
            Ok(ConfirmationResolver::new(
                ctx,
                *self.id(),
                Some(success),
                message.as_deref(),
                recommended_actions,
                self.func_id(),
                *func_binding.id(),
                context,
            )
            .await?)
        }
    }

    standard_model_accessor!(name, String, ConfirmationPrototypeResult);
    standard_model_accessor!(
        success_description,
        Option<String>,
        ConfirmationPrototypeResult
    );
    standard_model_accessor!(
        failure_description,
        Option<String>,
        ConfirmationPrototypeResult
    );
    standard_model_accessor!(provider, Option<String>, ConfirmationPrototypeResult);
    standard_model_accessor!(func_id, Pk(FuncId), ConfirmationPrototypeResult);
    standard_model_accessor!(schema_id, Pk(SchemaId), ConfirmationPrototypeResult);
    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        ConfirmationPrototypeResult
    );
    standard_model_accessor!(component_id, Pk(ComponentId), ConfirmationPrototypeResult);

    #[allow(clippy::too_many_arguments)]
    pub async fn list_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ConfirmationPrototypeResult<Vec<Self>> {
        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        let schema = component
            .schema(ctx)
            .await?
            .ok_or_else(|| ComponentError::NoSchema(*component.id()))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or_else(|| ComponentError::NoSchemaVariant(*component.id()))?;
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_CONTEXT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    schema_variant.id(),
                    schema.id(),
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }
}

impl_prototype_list_for_func! {model: ConfirmationPrototype}

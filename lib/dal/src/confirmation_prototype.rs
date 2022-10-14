use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::PgError;
use telemetry::prelude::*;

use crate::{
    func::backend::js_confirmation::{ConfirmationResult, FuncBackendJsConfirmationArgs},
    func::FuncId,
    impl_prototype_list_for_func, impl_standard_model, pk,
    prototype_context::{HasPrototypeContext, PrototypeContext},
    standard_model, standard_model_accessor, ActionPrototype, ActionPrototypeError, Component,
    ComponentError, ComponentId, ConfirmationResolver, ConfirmationResolverContext,
    ConfirmationResolverError, DalContext, FuncBinding, FuncBindingError, HistoryEventError,
    SchemaId, SchemaVariantId, StandardModel, StandardModelError, SystemId, Timestamp, Visibility,
    WriteTenancy,
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
}

pub type ConfirmationPrototypeResult<T> = Result<T, ConfirmationPrototypeError>;

const LIST_FOR_CONTEXT: &str = include_str!("queries/confirmation_prototype_list_for_context.sql");
const GET_BY_COMPONENT_AND_NAME: &str =
    include_str!("queries/confirmation_prototype_get_by_component_and_name.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationPrototypeContext {
    pub component_id: ComponentId,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub system_id: SystemId,
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

    fn system_id(&self) -> SystemId {
        self.system_id
    }

    fn set_system_id(&mut self, system_id: SystemId) {
        self.system_id = system_id;
    }
}

impl ConfirmationPrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
            system_id: SystemId::NONE,
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
    func_id: FuncId,
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
        context.set_system_id(self.system_id);

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
                "SELECT object FROM confirmation_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &func_id,
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

    pub async fn run(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ConfirmationPrototypeResult<ConfirmationResolver> {
        let args = FuncBackendJsConfirmationArgs {
            component: Component::view(ctx, component_id, system_id).await?.into(),
        };

        let json_args = serde_json::to_value(args)?;
        let (func_binding, func_binding_return_value, _created) =
            FuncBinding::find_or_create_and_execute(ctx, json_args, self.func_id()).await?;

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
                    system_id,
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
        context.set_system_id(system_id);
        if let Some(mut resolver) =
            ConfirmationResolver::find_for_prototype(ctx, self.id(), context.clone())
                .await?
                .pop()
        {
            resolver.set_success(ctx, success).await?;
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
                success,
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
    standard_model_accessor!(func_id, Pk(FuncId), ConfirmationPrototypeResult);
    standard_model_accessor!(schema_id, Pk(SchemaId), ConfirmationPrototypeResult);
    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        ConfirmationPrototypeResult
    );
    standard_model_accessor!(component_id, Pk(ComponentId), ConfirmationPrototypeResult);

    standard_model_accessor!(system_id, Pk(SystemId), ConfirmationPrototypeResult);

    #[allow(clippy::too_many_arguments)]
    pub async fn list_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ConfirmationPrototypeResult<Vec<Self>> {
        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        let schema = component
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_FOR_CONTEXT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &system_id,
                    schema_variant.id(),
                    schema.id(),
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    #[instrument(skip_all)]
    pub async fn get_by_component_and_name(
        ctx: &DalContext,
        component_id: ComponentId,
        name: impl AsRef<str>,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        system_id: SystemId,
    ) -> ConfirmationPrototypeResult<Option<Self>> {
        let name = name.as_ref();
        let maybe_row = ctx
            .txns()
            .pg()
            .query_opt(
                GET_BY_COMPONENT_AND_NAME,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &name,
                    &schema_id,
                    &schema_variant_id,
                    &system_id,
                ],
            )
            .await?;
        Ok(standard_model::option_object_from_row(maybe_row)?)
    }
}

impl_prototype_list_for_func! {model: ConfirmationPrototype}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn context_builder() {
        let mut c = ConfirmationPrototypeContext::new();
        c.set_component_id(22.into());
        assert_eq!(c.component_id(), 22.into());
    }
}

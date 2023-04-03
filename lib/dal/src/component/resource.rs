//! This module contains the ability to work with "resources" for [`Components`](crate::Component).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use veritech_client::ResourceStatus;

use crate::attribute::context::AttributeContextBuilder;
use crate::attribute::value::AttributeValue;
use crate::attribute::value::AttributeValueError;
use crate::component::ComponentResult;
use crate::func::binding_return_value::FuncBindingReturnValue;
use crate::ws_event::WsEvent;
use crate::{
    func::backend::js_command::CommandRunResult, ActionPrototype, AttributeReadContext,
    ChangeSetPk, Component, ComponentError, ComponentId, DalContext, SchemaVariant, StandardModel,
    WorkflowRunner, WsPayload,
};
use crate::{RootPropChild, WsEventResult};

impl Component {
    /// Calls [`Self::resource_by_id`] using the [`ComponentId`](Component) off [`Component`].
    pub async fn resource(&self, ctx: &DalContext) -> ComponentResult<CommandRunResult> {
        Self::resource_by_id(ctx, self.id).await
    }

    /// Find the object corresponding to "/root/resource".
    pub async fn resource_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<CommandRunResult> {
        let attribute_value = Self::resource_attribute_value_by_id(ctx, component_id).await?;

        let func_binding_return_value =
            FuncBindingReturnValue::get_by_id(ctx, &attribute_value.func_binding_return_value_id())
                .await?
                .ok_or_else(|| {
                    ComponentError::FuncBindingReturnValueNotFound(
                        attribute_value.func_binding_return_value_id(),
                    )
                })?;

        let value = func_binding_return_value
            .value()
            .map(|value| {
                if value == &serde_json::json!({}) {
                    return serde_json::json!({
                        "status": "ok",
                    });
                }
                value.clone()
            })
            .unwrap_or_else(|| {
                serde_json::json!({
                    "status": "ok",
                })
            });
        let result = CommandRunResult::deserialize(&value)?;
        Ok(result)
    }

    pub async fn resource_attribute_value_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<AttributeValue> {
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;
        let implicit_internal_provider = SchemaVariant::find_root_child_implicit_internal_provider(
            ctx,
            schema_variant_id,
            RootPropChild::Resource,
        )
        .await?;

        let value_context = AttributeReadContext {
            internal_provider_id: Some(*implicit_internal_provider.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };

        let attribute_value = AttributeValue::find_for_context(ctx, value_context)
            .await?
            .ok_or(ComponentError::AttributeValueNotFoundForContext(
                value_context,
            ))?;
        Ok(attribute_value)
    }

    /// Sets the "string" field, "/root/resource" with a given value. After that, ensure dependent
    /// [`AttributeValues`](crate::AttributeValue) are updated.
    ///
    /// Returns "true" if the resource tree has been updated. Returns "false" if the cached
    /// value is used.
    pub async fn set_resource(
        &self,
        ctx: &DalContext,
        result: CommandRunResult,
        trigger_dependent_values_update: bool,
    ) -> ComponentResult<bool> {
        if ctx.visibility().change_set_pk != ChangeSetPk::NONE {
            return Err(ComponentError::CannotUpdateResourceTreeInChangeSet);
        }

        let resource_attribute_value = Component::root_prop_child_attribute_value_for_component(
            ctx,
            self.id,
            RootPropChild::Resource,
        )
        .await?;

        let root_attribute_value = resource_attribute_value
            .parent_attribute_value(ctx)
            .await?
            .ok_or_else(|| AttributeValueError::ParentNotFound(*resource_attribute_value.id()))?;

        let update_attribute_context =
            AttributeContextBuilder::from(resource_attribute_value.context)
                .set_component_id(self.id)
                .to_context()?;

        if trigger_dependent_values_update {
            let (_, _) = AttributeValue::update_for_context(
                ctx,
                *resource_attribute_value.id(),
                Some(*root_attribute_value.id()),
                update_attribute_context,
                Some(serde_json::to_value(result)?),
                None,
            )
            .await?;
        } else {
            // Jacob / Paulo / Victor / Paul:
            // We use this func to stop enqueueing another DependentValuesUpdate job
            // The fix job was running DependentValuesUpdate inline and this func was also
            // queueing a DependentValuesUpdate.
            let (_, _) = AttributeValue::update_for_context_without_propagating_dependent_values(
                ctx,
                *resource_attribute_value.id(),
                Some(*root_attribute_value.id()),
                update_attribute_context,
                Some(serde_json::to_value(result)?),
                None,
            )
            .await?;
        }
        Ok(true)
    }

    pub async fn act(&self, ctx: &DalContext, action_name: &str) -> ComponentResult<()> {
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;
        let action = match ActionPrototype::find_by_name(
            ctx,
            action_name,
            *schema.id(),
            *schema_variant.id(),
        )
        .await?
        {
            Some(action) => action,
            None => return Ok(()),
        };

        let prototype = action.workflow_prototype(ctx).await?;
        let run_id: usize = rand::random();
        let (_runner, _state, _func_binding_return_values, _resources) =
            WorkflowRunner::run(ctx, run_id, *prototype.id(), self.id).await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceView {
    pub status: ResourceStatus,
    pub message: Option<String>,
    pub data: Option<Value>,
    pub logs: Vec<String>,
    pub last_synced: Option<String>,
}

impl ResourceView {
    pub fn new(result: CommandRunResult) -> Self {
        Self {
            data: result.value,
            message: result.message,
            status: result.status,
            logs: result.logs,
            last_synced: result.last_synced,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRefreshId {
    component_id: ComponentId,
}

impl WsEvent {
    pub async fn resource_refreshed(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ResourceRefreshed(ResourceRefreshId { component_id }),
        )
        .await
    }
}

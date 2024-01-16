//! This module contains the ability to work with "resources" for [`Components`](crate::Component).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use veritech_client::ResourceStatus;

use crate::attribute::context::AttributeContextBuilder;
use crate::attribute::value::AttributeValue;
use crate::attribute::value::AttributeValueError;
use crate::component::ComponentResult;
use crate::func::binding_return_value::FuncBindingReturnValue;
use crate::ws_event::WsEvent;
use crate::{
    func::backend::js_action::ActionRunResult, ActionKind, ActionPrototype, ActionPrototypeContext,
    AttributeReadContext, Component, ComponentError, ComponentId, DalContext, SchemaVariant,
    StandardModel, WsPayload,
};
use crate::{RootPropChild, WsEventResult};

impl Component {
    /// Calls [`Self::resource_by_id`] using the [`ComponentId`](Component) off [`Component`].
    pub async fn resource(&self, ctx: &DalContext) -> ComponentResult<ActionRunResult> {
        Self::resource_by_id(ctx, self.id).await
    }

    /// Find the object corresponding to "/root/resource".
    pub async fn resource_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<ActionRunResult> {
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
        let result = ActionRunResult::deserialize(&value)?;
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
        result: ActionRunResult,
    ) -> ComponentResult<bool> {
        self.set_resource_raw(ctx, result, true).await
    }

    pub async fn set_resource_raw(
        &self,
        ctx: &DalContext,
        result: ActionRunResult,
        check_change_set: bool,
    ) -> ComponentResult<bool> {
        let ctx = &ctx.clone_without_deleted_visibility();

        if check_change_set && !ctx.visibility().is_head() {
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

        let (_, _) = AttributeValue::update_for_context(
            ctx,
            *resource_attribute_value.id(),
            Some(*root_attribute_value.id()),
            update_attribute_context,
            Some(serde_json::to_value(result)?),
            None,
        )
        .await?;
        Ok(true)
    }

    pub async fn act(&self, ctx: &DalContext, action: ActionKind) -> ComponentResult<()> {
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;

        let action = match ActionPrototype::find_for_context_and_kind(
            ctx,
            action,
            ActionPrototypeContext {
                schema_variant_id: *schema_variant.id(),
            },
        )
        .await?
        .pop()
        {
            Some(action) => action,
            None => return Ok(()),
        };

        action.run(ctx, *self.id()).await?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceView {
    pub status: Option<ResourceStatus>,
    pub message: Option<String>,
    pub data: Option<Value>,
    pub logs: Vec<String>,
    pub last_synced: Option<String>,
}

impl ResourceView {
    pub fn new(result: ActionRunResult) -> Self {
        Self {
            data: result.payload,
            message: result.message,
            status: result.status,
            logs: result.logs,
            last_synced: result.last_synced,
        }
    }

    /// Generate a map of [views](Self) for all [`Components`](Component) in the workspace.
    pub async fn list_with_deleted(
        ctx: &DalContext,
    ) -> ComponentResult<HashMap<ComponentId, Self>> {
        let ctx = &ctx.clone_with_delete_visibility();
        let mut resources = HashMap::new();
        for component in Component::list(ctx).await? {
            if !component.is_destroyed() {
                // Use the entry API to ensure that we do not process the same component twice, if
                // duplicates were accidentally(?) provided.
                resources
                    .entry(*component.id())
                    .or_insert(Self::new(component.resource(ctx).await?));
            }
        }
        Ok(resources)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRefreshedPayload {
    component_id: ComponentId,
}

impl WsEvent {
    pub async fn resource_refreshed(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ResourceRefreshed(ResourceRefreshedPayload { component_id }),
        )
        .await
    }
}

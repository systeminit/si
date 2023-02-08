use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use telemetry::prelude::*;

use crate::attribute::value::AttributeValue;
use crate::component::{
    ComponentResult, LIST_ALL_RESOURCE_IMPLICIT_INTERNAL_PROVIDER_ATTRIBUTE_VALUES,
};
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::job::definition::DependentValuesUpdate;
use crate::schema::variant::SchemaVariantError;
use crate::ws_event::WsEvent;
use crate::{
    standard_model, ActionPrototype, ActionPrototypeError, AttributeReadContext,
    AttributeValueError, AttributeValueId, ComponentError, DalContext, FuncBindingReturnValue,
    FuncDescription, FuncDescriptionContents, FuncId, Node, NodeError, RootPropChild, SchemaId,
    SchemaVariant, SchemaVariantId, StandardModel, WsEventResult, WsPayload,
};
use crate::{Component, ComponentId};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfirmationStatusView {
    Running,
    Failure,
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationView {
    pub attribute_value_id: AttributeValueId,
    func_id: FuncId,
    func_binding_return_value_id: FuncBindingReturnValueId,
    title: String,
    pub component_id: ComponentId,
    schema_variant_id: SchemaVariantId,
    pub schema_id: SchemaId,
    description: Option<String>,
    output: Option<Vec<String>>,
    recommended_actions_raw: Vec<String>,
    pub status: ConfirmationStatusView,
    pub provider: Option<String>,
}

impl ConfirmationView {
    /// List all recommended [`actions`](crate::ActionPrototype) via the raw actions off
    /// [`self`](Self).
    pub async fn recommended_actions(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<ActionPrototype>> {
        let mut recommended_actions = Vec::new();
        for action in &self.recommended_actions_raw {
            let action_prototype =
                ActionPrototype::find_by_name(ctx, action, self.schema_id, self.schema_variant_id)
                    .await?
                    .ok_or_else(|| ActionPrototypeError::NotFoundByName(action.clone()))?;
            recommended_actions.push(action_prototype);
        }
        Ok(recommended_actions)
    }
}

// TODO(nick): use this for listing confirmations, like qualifications in the future.
// FIXME(nick): use the formal types from the new version of function authoring instead of this
// struct. This struct is a temporary stopgap until that's implemented.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationEntry {
    pub success: Option<bool>,
    #[serde(default)]
    pub recommended_actions: Vec<String>,
}

impl Component {
    /// List all [`AttributeValues`](crate::AttributeValue) whose
    /// [`AttributeContext`](crate::AttributeContext) contains a populated [`ComponentId`](Self)
    /// and a populated [`InternalProviderId`](crate::InternalProvider) where the latter is the
    /// ID for the _implicit_ [`InternalProvider`](crate::InternalProvider) corresponding to
    /// "/root/resource" (child of [`RootProp`](crate::RootProp).
    ///
    /// In other words, this query should find as many [`AttributeValues`](crate::AttributeValue)
    /// as there are [`Components`](Self) in the workspace.
    #[instrument(skip_all)]
    pub async fn list_all_resource_implicit_internal_provider_attribute_values(
        ctx: &DalContext,
    ) -> ComponentResult<Vec<AttributeValue>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_ALL_RESOURCE_IMPLICIT_INTERNAL_PROVIDER_ATTRIBUTE_VALUES,
                &[ctx.tenancy(), ctx.visibility()],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Run confirmations for all [`Components`](Self) in the workspace by running a
    /// [`DependentValuesUpdate`](crate::job::definition::DependentValuesUpdate) job for every
    /// [`AttributeValue`](crate::AttributeValue) corresponding to the "/root/resource" implicit
    /// [`InternalProvider`](crate::InternalProvider) for every [`Component`](crate::Component).
    pub async fn run_all_confirmations(ctx: &DalContext) -> ComponentResult<()> {
        let resource_attribute_values =
            Component::list_all_resource_implicit_internal_provider_attribute_values(ctx).await?;

        ctx.enqueue_job(DependentValuesUpdate::new(
            ctx,
            resource_attribute_values
                .iter()
                .map(|av| *av.id())
                .collect::<Vec<AttributeValueId>>(),
        ))
        .await;

        WsEvent::ran_confirmations(ctx).await?;

        Ok(())
    }

    // TODO(nick): big query potential here.
    #[instrument(skip_all)]
    pub async fn list_confirmations(ctx: &DalContext) -> ComponentResult<Vec<ConfirmationView>> {
        let sorted_node_ids =
            Node::list_topologically_ish_sorted_configuration_nodes(ctx, false).await?;
        let mut results = Vec::new();

        for sorted_node_id in sorted_node_ids {
            let sorted_node = Node::get_by_id(ctx, &sorted_node_id)
                .await?
                .ok_or(NodeError::NotFound(sorted_node_id))?;
            let component = sorted_node
                .component(ctx)
                .await?
                .ok_or(NodeError::ComponentIsNone)?;
            let schema_variant = component
                .schema_variant(ctx)
                .await?
                .ok_or_else(|| ComponentError::NoSchemaVariant(*component.id()))?;
            let schema = schema_variant
                .schema(ctx)
                .await?
                .ok_or_else(|| SchemaVariantError::MissingSchema(*schema_variant.id()))?;

            // Prepare to assemble qualification views and access the "/root/qualification" prop tree.
            // We will use its implicit internal provider id and its corresponding prop id to do so.
            let confirmation_map_implicit_internal_provider =
                SchemaVariant::find_root_child_implicit_internal_provider(
                    ctx,
                    *schema_variant.id(),
                    RootPropChild::Confirmation,
                )
                .await?;

            // Collect all the func binding return value ids for the child attribute values
            // (map entries) for reference later.
            let confirmation_map_prop_attribute_read_context = AttributeReadContext {
                prop_id: Some(*confirmation_map_implicit_internal_provider.prop_id()),
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            };
            let confirmation_map_prop_attribute_value =
                AttributeValue::find_for_context(ctx, confirmation_map_prop_attribute_read_context)
                    .await?
                    .ok_or(AttributeValueError::NotFoundForReadContext(
                        confirmation_map_prop_attribute_read_context,
                    ))?;

            // Collect all the information for the map entries (child attribute values) that we will
            // need for assembling views later.
            let mut entry_attribute_values: HashMap<
                String,
                (FuncBindingReturnValueId, AttributeValueId, FuncId),
            > = HashMap::new();
            for entry_attribute_value in confirmation_map_prop_attribute_value
                .child_attribute_values(ctx)
                .await?
            {
                let entry_attribute_value_id = *entry_attribute_value.id();
                let func_binding_return_value_id =
                    entry_attribute_value.func_binding_return_value_id();
                let attribute_prototype = entry_attribute_value
                    .attribute_prototype(ctx)
                    .await?
                    .ok_or_else(|| {
                        ComponentError::MissingAttributePrototype(*entry_attribute_value.id())
                    })?;
                let key =
                    entry_attribute_value
                        .key
                        .ok_or(ComponentError::FoundMapEntryWithoutKey(
                            entry_attribute_value_id,
                        ))?;
                entry_attribute_values.insert(
                    key,
                    (
                        func_binding_return_value_id,
                        entry_attribute_value_id,
                        attribute_prototype.func_id(),
                    ),
                );
            }

            // Now, find all confirmations in the tree.
            let confirmation_map_implicit_attribute_read_context = AttributeReadContext {
                internal_provider_id: Some(*confirmation_map_implicit_internal_provider.id()),
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            };
            let confirmation_map_implicit_attribute_value = AttributeValue::find_for_context(
                ctx,
                confirmation_map_implicit_attribute_read_context,
            )
            .await?
            .ok_or(AttributeValueError::NotFoundForReadContext(
                confirmation_map_implicit_attribute_read_context,
            ))?;
            let maybe_confirmation_map_value = confirmation_map_implicit_attribute_value
                .get_value(ctx)
                .await?;
            if let Some(confirmation_map_value) = maybe_confirmation_map_value {
                let confirmation_map: HashMap<String, ConfirmationEntry> =
                    serde_json::from_value(confirmation_map_value)?;

                for (confirmation_name, entry) in confirmation_map {
                    let (
                        found_func_binding_return_value_id,
                        found_attribute_value_id,
                        found_func_id,
                    ) = entry_attribute_values
                        .get(&confirmation_name)
                        .ok_or_else(|| {
                            ComponentError::MissingFuncBindingReturnValueIdForLeafEntryName(
                                confirmation_name.clone(),
                            )
                        })?;

                    // Collect the output from the func binding return value.
                    let mut output = Vec::new();
                    if let Some(func_binding_return_value) =
                        FuncBindingReturnValue::get_by_id(ctx, found_func_binding_return_value_id)
                            .await?
                    {
                        if let Some(output_streams) =
                            func_binding_return_value.get_output_stream(ctx).await?
                        {
                            for output_stream in output_streams {
                                output.push(output_stream.message);
                            }
                        }
                    }

                    // Determine the status based on the entry's current value.
                    let status = match entry.success {
                        Some(true) => ConfirmationStatusView::Success,
                        Some(false) => ConfirmationStatusView::Failure,
                        None => ConfirmationStatusView::Running,
                    };

                    // Dynamically determine the description based on the status.
                    let (description, maybe_title, maybe_provider) =
                        match FuncDescription::find_for_func_and_schema_variant(
                            ctx,
                            *found_func_id,
                            *schema_variant.id(),
                        )
                        .await?
                        {
                            Some(description) => match description.deserialized_contents()? {
                                FuncDescriptionContents::Confirmation {
                                    success_description,
                                    failure_description,
                                    name,
                                    provider,
                                } => match status {
                                    ConfirmationStatusView::Success => {
                                        (success_description, Some(name), provider)
                                    }
                                    ConfirmationStatusView::Failure => {
                                        (failure_description, Some(name), provider)
                                    }
                                    _ => (None, Some(name), provider),
                                },
                            },
                            None => (None, None, None),
                        };

                    // Assemble the view.
                    let view = ConfirmationView {
                        attribute_value_id: *found_attribute_value_id,
                        func_id: *found_func_id,
                        func_binding_return_value_id: *found_func_binding_return_value_id,
                        title: match maybe_title {
                            Some(title) => title,
                            None => confirmation_name,
                        },
                        component_id: *component.id(),
                        schema_variant_id: *schema_variant.id(),
                        schema_id: *schema.id(),
                        description,
                        output: Some(output.clone()).filter(|o| !o.is_empty()),
                        recommended_actions_raw: entry.recommended_actions,
                        status,
                        provider: maybe_provider,
                    };
                    results.push(view);
                }
            }
        }

        Ok(results)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationRunPayload {
    success: bool,
}

impl WsEvent {
    pub async fn ran_confirmations(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::RanConfirmations(ConfirmationRunPayload { success: true }),
        )
        .await
    }
}

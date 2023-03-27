use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;

use crate::action_prototype::ActionKind;
use crate::attribute::value::AttributeValue;
use crate::component::confirmation::view::{ConfirmationView, PrimaryActionKind};
use crate::component::{
    ComponentResult, LIST_ALL_RESOURCE_IMPLICIT_INTERNAL_PROVIDER_ATTRIBUTE_VALUES,
};
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::job::definition::DependentValuesUpdate;
use crate::{
    standard_model, AttributeReadContext, AttributeValueError, AttributeValueId, ComponentError,
    DalContext, Fix, FuncId, Node, NodeError, RootPropChild, Schema, SchemaVariant, StandardModel,
    WsEvent, WsEventResult, WsPayload,
};
use crate::{Component, ComponentId};

pub mod view;

/// Cache metadata for confirmations. The "key" refers to the literal "key" in the
/// "/root/confirmations" map entry.
type ConfirmationMetadataCache =
    HashMap<String, (FuncBindingReturnValueId, AttributeValueId, FuncId)>;

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
            .await?
            .pg()
            .query(
                LIST_ALL_RESOURCE_IMPLICIT_INTERNAL_PROVIDER_ATTRIBUTE_VALUES,
                &[ctx.tenancy(), &ctx.visibility().to_deleted()],
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
            ctx.access_builder(),
            *ctx.visibility(),
            resource_attribute_values
                .iter()
                .map(|av| *av.id())
                .collect::<Vec<AttributeValueId>>(),
        ))
        .await?;

        Ok(())
    }

    // TODO(nick): big query potential here.
    pub async fn list_confirmations(ctx: &DalContext) -> ComponentResult<Vec<ConfirmationView>> {
        let sorted_node_ids =
            Node::list_topologically_sorted_configuration_nodes_with_stable_ordering(ctx, false)
                .await?;

        // Go through all sorted nodes, assemble confirmations and order them by primary action
        // kind.
        let mut delete_results = Vec::new();
        let mut create_results = Vec::new();
        let mut other_results = Vec::new();
        let mut no_recommendation_results = Vec::new();

        let ctx_with_deleted = &ctx.clone_with_delete_visibility();
        for sorted_node_id in sorted_node_ids {
            let sorted_node = Node::get_by_id(ctx_with_deleted, &sorted_node_id)
                .await?
                .ok_or(NodeError::NotFound(sorted_node_id))?;
            let component = sorted_node
                .component(ctx_with_deleted)
                .await?
                .ok_or(NodeError::ComponentIsNone)?;

            if component.visibility.deleted_at.is_some() && !component.needs_destroy() {
                continue;
            }
            if let Some((component_specific_confirmations, primary_action_kind)) =
                Self::list_confirmations_for_component(ctx, *component.id()).await?
            {
                match primary_action_kind {
                    PrimaryActionKind::HasRecommendations(action_kind) => match action_kind {
                        ActionKind::Create => {
                            create_results.extend(component_specific_confirmations)
                        }
                        ActionKind::Other => other_results.extend(component_specific_confirmations),
                        ActionKind::Destroy => {
                            delete_results.extend(component_specific_confirmations)
                        }
                    },
                    PrimaryActionKind::NoRecommendations => {
                        no_recommendation_results.extend(component_specific_confirmations)
                    }
                }
            }
        }

        // We need to invert the order of the delete results before we create the final results.
        // The final results are in the following order: destroy, create, other and "no
        // recommendations" based on a topological sort of the nodes.
        let mut results = Vec::new();
        delete_results.reverse();
        results.extend(delete_results);
        results.extend(create_results);
        results.extend(other_results);
        results.extend(no_recommendation_results);
        Ok(results)
    }

    async fn list_confirmations_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Option<(Vec<ConfirmationView>, PrimaryActionKind)>> {
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;
        let schema_id = Self::schema_id(ctx, component_id).await?;
        let schema_name = Schema::get_by_id(ctx, &schema_id)
            .await?
            .ok_or(ComponentError::NoSchema(component_id))?
            .name()
            .to_string();

        // Refresh running fixes.
        // FIXME(paulo,fletcher,nick,paul): hardcoding 3 minutes timeout to avoid permanently running fixes
        let fixes = Fix::find_by_attr_null(ctx, "finished_at").await?;
        let mut running_fixes = Vec::new();
        for fix in fixes {
            if Utc::now().signed_duration_since(fix.timestamp().created_at)
                < chrono::Duration::minutes(3)
            {
                running_fixes.push(fix);
            }
        }

        let (all_confirmations_attribute_value, cache) =
            Self::prepare_confirmations(ctx, component_id).await?;

        match all_confirmations_attribute_value.get_value(ctx).await? {
            Some(all_confirmations_raw) => {
                let deserialized_value: HashMap<String, ConfirmationEntry> =
                    serde_json::from_value(all_confirmations_raw)?;
                let view = ConfirmationView::assemble_for_component(
                    ctx,
                    component_id,
                    &deserialized_value,
                    &cache,
                    schema_id,
                    schema_variant_id,
                    &running_fixes,
                    schema_name.clone(),
                )
                .await
                .map_err(|e| ComponentError::ConfirmationView(e.to_string()))?;
                Ok(Some(view))
            }
            None => Ok(None),
        }
    }

    /// Find the [`AttributeValue`](crate::AttributeValue) corresponding to the _implicit_
    /// [`InternalProvider`](crate::InternalProvider) corresponding to the "/root/confirmation"
    /// [`Prop`](crate::Prop). Additionally, a [`cache`](ConfirmationMetadataCache) is returned,
    /// which is used to reference values when assembling [`ConfirmationViews`](ConfirmationView).
    ///
    /// Why is the cache important? When looking at the current state of "/root/confirmation" for a
    /// [`Component`], it is likely best to look at the literal [`Value`](serde_json::Value) of the
    /// map's [`AttributeValue`](crate::AttributeValue). Well, what if you want some metadata or
    /// additional information for a specific confirmation within that [`Value`](serde_json::Value),
    /// you will not be able to access it. Thus, you can use the confirmation's name as a "key"
    /// to access corresponding metadata from the cache.
    async fn prepare_confirmations(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(AttributeValue, ConfirmationMetadataCache)> {
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;

        // Prepare to assemble qualification views and access the "/root/confirmation" prop
        // tree. We will use its implicit internal provider id and its corresponding prop id to
        // do so.
        let confirmation_map_implicit_internal_provider =
            SchemaVariant::find_root_child_implicit_internal_provider(
                ctx,
                schema_variant_id,
                RootPropChild::Confirmation,
            )
            .await?;

        // Collect all the func binding return value ids for the child attribute values
        // (map entries) for reference later.
        let confirmation_map_prop_attribute_read_context = AttributeReadContext {
            prop_id: Some(*confirmation_map_implicit_internal_provider.prop_id()),
            component_id: Some(component_id),
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
        let mut confirmation_metadata_cache: HashMap<
            String,
            (FuncBindingReturnValueId, AttributeValueId, FuncId),
        > = HashMap::new();
        for entry_attribute_value in confirmation_map_prop_attribute_value
            .child_attribute_values(ctx)
            .await?
        {
            let entry_attribute_value_id = *entry_attribute_value.id();
            let func_binding_return_value_id = entry_attribute_value.func_binding_return_value_id();
            let attribute_prototype = entry_attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or_else(|| {
                    ComponentError::MissingAttributePrototype(*entry_attribute_value.id())
                })?;
            let key = entry_attribute_value
                .key
                .ok_or(ComponentError::FoundMapEntryWithoutKey(
                    entry_attribute_value_id,
                ))?;
            confirmation_metadata_cache.insert(
                key,
                (
                    func_binding_return_value_id,
                    entry_attribute_value_id,
                    attribute_prototype.func_id(),
                ),
            );
        }

        // Now, find the attribute value corresponding to the implicit internal provider for the
        // entire confirmations map. We'll need it to get the current state of the confirmations.
        let confirmation_map_implicit_attribute_read_context = AttributeReadContext {
            internal_provider_id: Some(*confirmation_map_implicit_internal_provider.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };
        let confirmation_map_implicit_attribute_value =
            AttributeValue::find_for_context(ctx, confirmation_map_implicit_attribute_read_context)
                .await?
                .ok_or(AttributeValueError::NotFoundForReadContext(
                    confirmation_map_implicit_attribute_read_context,
                ))?;

        Ok((
            confirmation_map_implicit_attribute_value,
            confirmation_metadata_cache,
        ))
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationsUpdatedPayload {
    success: bool,
}

impl WsEvent {
    pub async fn confirmations_updated(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ConfirmationsUpdated(ConfirmationsUpdatedPayload { success: true }),
        )
        .await
    }
}

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;

use crate::action_prototype::ActionKind;
use crate::attribute::value::AttributeValue;
use crate::component::{
    ComponentResult, LIST_ALL_RESOURCE_IMPLICIT_INTERNAL_PROVIDER_ATTRIBUTE_VALUES,
};
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::job::definition::DependentValuesUpdate;
use crate::{
    standard_model, ActionPrototype, ActionPrototypeError, AttributeReadContext,
    AttributeValueError, AttributeValueId, ComponentError, DalContext, Fix, FixResolver,
    FuncBindingReturnValue, FuncDescription, FuncDescriptionContents, FuncId, Node, NodeError,
    RootPropChild, Schema, SchemaId, SchemaVariant, SchemaVariantId, StandardModel, WsEvent,
    WsEventResult, WsPayload,
};
use crate::{Component, ComponentId};

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfirmationStatus {
    Running,
    Failure,
    Success,
    // FIXME(nick,paulo,paul,wendy): probably remove this once the fix flow is working again.
    NeverStarted,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationView {
    pub attribute_value_id: AttributeValueId,
    pub title: String,
    description: Option<String>,

    pub schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    pub component_id: ComponentId,

    schema_name: String,
    component_name: String,

    output: Option<Vec<String>>,
    pub status: ConfirmationStatus,
    pub provider: Option<String>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    // TODO(nick,paulo,paul,wendy): yes, these fields are technically already on the confirmation
    // itself and we could just map the recommendations back to the confirmations in the frontend,
    // but we want shit to work again before optimizing. Fix the fix flow!
    pub confirmation_attribute_value_id: AttributeValueId,
    pub component_id: ComponentId,
    component_name: String,
    provider: Option<String>,

    /// The title of a [recommendation](Self). An example: "Create EC2 Instance".
    name: String,
    /// Maps to the name of an [`ActionPrototype`](crate::ActionPrototype). An example: "create".
    pub recommended_action: String,
    /// The [`kind`](crate::action_prototype::ActionKind) of
    /// [`ActionPrototype`](crate::ActionPrototype) that the recommended
    /// [action](crate::ActionPrototype) corresponds to.
    action_kind: ActionKind,
    /// The last recorded [`status`](RecommendationStatus) of the [recommendation](Self).
    pub status: RecommendationStatus,
    /// Indicates the ability to "run" the [`Fix`](crate::Fix) associated with the
    /// [recommendation](Self).
    is_runnable: RecommendationIsRunnable,
}

/// Tracks the last known status of a [`Recommendation`] (corresponds to the
/// [`FixResolver`](crate::FixResolver)).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RecommendationStatus {
    /// The last execution of the [`Recommendation`] succeeded.
    Success,
    /// The last execution of the [`Recommendation`] failed.
    Failure,
    /// The last execution of the [`Recommendation`] is still running.
    Running,
    /// The [`Recommendation`] has never been ran.
    Unstarted,
}

/// Tracks the ability to run a [`Recommendation`] (corresponds to the state of "/root/resource"
/// and the [`ActionKind`](crate::action_prototype::ActionKind) on the corresponding
/// [`ActionPrototype`](crate::ActionPrototype).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RecommendationIsRunnable {
    /// The [`Recommendation`] is ready to be ran.
    Yes,
    /// The [`Recommendation`] is not ready to be ran.
    No,
    /// There is a [`Fix`](crate::Fix) in-flight that prevents the [`Recommendation`] from being
    /// able to be ran.
    Running,
}

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

        Ok(())
    }

    // TODO(nick): big query potential here.
    pub async fn list_confirmations(ctx: &DalContext) -> ComponentResult<Vec<ConfirmationView>> {
        let sorted_node_ids =
            Node::list_topologically_ish_sorted_configuration_nodes(ctx, false).await?;
        let mut results = Vec::new();

        let ctx_with_deleted = &ctx.clone_with_delete_visibility();

        for sorted_node_id in sorted_node_ids {
            let sorted_node = Node::get_by_id(ctx_with_deleted, &sorted_node_id)
                .await?
                .ok_or(NodeError::NotFound(sorted_node_id))?;
            let component = sorted_node
                .component(ctx_with_deleted)
                .await?
                .ok_or(NodeError::ComponentIsNone)?;
            let component_specific_confirmations =
                Self::list_confirmations_for_component(ctx, *component.id()).await?;
            results.extend(component_specific_confirmations);
        }

        Ok(results)
    }

    async fn list_confirmations_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<ConfirmationView>> {
        let mut results = Vec::new();
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
        for mut fix in fixes {
            if Utc::now().signed_duration_since(fix.timestamp().created_at)
                > chrono::Duration::minutes(3)
            {
                fix.set_finished_at(ctx, Some(Utc::now().to_string()))
                    .await
                    .map_err(Box::new)?;
            } else {
                running_fixes.push(fix);
            }
        }

        let (all_confirmations_attribute_value, cache) =
            Self::prepare_confirmations(ctx, component_id).await?;

        let all_confirmations: HashMap<String, ConfirmationEntry> =
            match all_confirmations_attribute_value.get_value(ctx).await? {
                Some(all_confirmations_raw) => {
                    let deserialized_value: HashMap<String, ConfirmationEntry> =
                        serde_json::from_value(all_confirmations_raw)?;
                    // TODO(nick,paulo,paul,wendy): decide what to do if confirmation entries are
                    // all empty.
                    //
                    // for entry in deserialized_value.values() {
                    //     if entry.success.is_none() && entry.recommended_actions.is_empty() {
                    //         let view = ConfirmationView {
                    //             attribute_value_id,
                    //             func_id,
                    //             func_binding_return_value_id,
                    //             title: key,
                    //             component_id,
                    //             schema_variant_id,
                    //             schema_id,
                    //             description: None,
                    //             output: None,
                    //             recommendations: vec![],
                    //             status: ConfirmationStatus::Failure,
                    //             provider: None,
                    //         };
                    //         results.push(view);
                    //     }
                    // }
                    deserialized_value
                }
                None => {
                    for (key, (_, attribute_value_id, _)) in cache {
                        let view = ConfirmationView {
                            attribute_value_id,
                            title: key,
                            component_id,
                            schema_variant_id,
                            schema_id,
                            schema_name: schema_name.clone(),
                            component_name: Component::find_name(ctx, component_id).await?,
                            description: None,
                            output: None,
                            recommendations: vec![],
                            status: ConfirmationStatus::NeverStarted,
                            provider: None,
                        };
                        results.push(view);
                    }
                    return Ok(results);
                }
            };

        for (confirmation_name, entry) in all_confirmations {
            let (found_func_binding_return_value_id, found_attribute_value_id, found_func_id) =
                cache.get(&confirmation_name).ok_or_else(|| {
                    ComponentError::MissingFuncBindingReturnValueIdForLeafEntryName(
                        confirmation_name.clone(),
                    )
                })?;

            // Collect the output from the func binding return value.
            let mut output = Vec::new();
            if let Some(func_binding_return_value) =
                FuncBindingReturnValue::get_by_id(ctx, found_func_binding_return_value_id).await?
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
            let confirmation_status = match entry.success {
                Some(true) => ConfirmationStatus::Success,
                Some(false) => ConfirmationStatus::Failure,
                None => {
                    // FIXME(nick,paulo,paul,wendy): in the past, the "None" state represented a
                    // running confirmation in order to prevent race conditions. We may or may not
                    // continue this behavior moving forward... we will ponder on this.
                    ConfirmationStatus::NeverStarted
                }
            };

            // Dynamically determine the description based on the status.
            let (description, maybe_title, maybe_provider) =
                match FuncDescription::find_for_func_and_schema_variant(
                    ctx,
                    *found_func_id,
                    schema_variant_id,
                )
                .await?
                {
                    Some(description) => match description.deserialized_contents()? {
                        FuncDescriptionContents::Confirmation {
                            success_description,
                            failure_description,
                            name,
                            provider,
                        } => match confirmation_status {
                            ConfirmationStatus::Success => {
                                (success_description, Some(name), provider)
                            }
                            ConfirmationStatus::Failure => {
                                (failure_description, Some(name), provider)
                            }
                            _ => (None, Some(name), provider),
                        },
                    },
                    None => (None, None, None),
                };

            let fix_resolver =
                FixResolver::find_for_confirmation_attribute_value(ctx, *found_attribute_value_id)
                    .await?;

            // Gather all the action prototypes from the recommended actions raw strings.
            let mut recommendations = Vec::new();
            for action in entry.recommended_actions {
                let action_prototype =
                    ActionPrototype::find_by_name(ctx, &action, schema_id, schema_variant_id)
                        .await?
                        .ok_or_else(|| ActionPrototypeError::NotFoundByName(action.clone()))?;

                // Check if a fix is running before gathering the last recorded status of the
                // recommendation and the ability to run the recommendation.
                let is_running = confirmation_status == ConfirmationStatus::Running
                    || running_fixes.iter().any(|r| {
                        r.component_id() == component_id && r.action() == action_prototype.name()
                    });

                // Track the last recorded status of the recommendation.
                let recommendation_status = if is_running {
                    RecommendationStatus::Running
                } else {
                    match fix_resolver.as_ref().and_then(FixResolver::success) {
                        Some(true) => RecommendationStatus::Success,
                        Some(false) => RecommendationStatus::Failure,
                        None => RecommendationStatus::Unstarted,
                    }
                };

                // Track the ability to run the recommendation.
                // TODO(nick): we do not need an enum here since "running" will be accurate for the
                // "is_running" boolean. We should consider replacing the enum with a boolean.
                let recommendation_is_runnable = if is_running {
                    RecommendationIsRunnable::Running
                } else {
                    let resource = Component::resource_by_id(ctx, component_id).await?;
                    match (action_prototype.kind(), resource.value) {
                        (ActionKind::Create, Some(_)) => RecommendationIsRunnable::No,
                        (ActionKind::Create, None) => RecommendationIsRunnable::Yes,
                        (ActionKind::Other, Some(_)) => RecommendationIsRunnable::Yes,
                        (ActionKind::Other, None) => RecommendationIsRunnable::No,
                        (ActionKind::Destroy, Some(_)) => RecommendationIsRunnable::Yes,
                        (ActionKind::Destroy, None) => RecommendationIsRunnable::No,
                    }
                };

                let workflow_prototype = action_prototype.workflow_prototype(ctx).await?;

                recommendations.push(Recommendation {
                    confirmation_attribute_value_id: *found_attribute_value_id,
                    component_id,
                    component_name: Component::find_name(ctx, component_id).await?,
                    provider: maybe_provider.clone(),
                    name: workflow_prototype.title().to_owned(),
                    recommended_action: action_prototype.name().to_owned(),
                    action_kind: action_prototype.kind().to_owned(),
                    status: recommendation_status,
                    is_runnable: recommendation_is_runnable,
                })
            }

            // Assemble the view.
            let view = ConfirmationView {
                attribute_value_id: *found_attribute_value_id,
                title: match maybe_title {
                    Some(title) => title,
                    None => confirmation_name,
                },
                component_id,
                schema_variant_id,
                schema_id,
                schema_name: schema_name.clone(),
                component_name: Component::find_name(ctx, component_id).await?,
                description,
                output: Some(output.clone()).filter(|o| !o.is_empty()),
                recommendations,
                status: confirmation_status,
                provider: maybe_provider,
            };
            results.push(view);
        }

        Ok(results)
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

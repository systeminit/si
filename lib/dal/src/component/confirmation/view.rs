//! This module contains [`ConfirmationView`] and everything related to it (including the ability
//! to assemble [`views`](ConfirmationView).

#![warn(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;

use crate::action_prototype::ActionKind;
use crate::component::confirmation::{ConfirmationEntry, ConfirmationMetadataCache};
use crate::fix::FixHistoryView;
use crate::{
    ActionPrototype, ActionPrototypeContext, ActionPrototypeError, ActionPrototypeId,
    AttributeValueId, ComponentError, DalContext, Fix, FixResolver, FixResolverError, Func,
    FuncBindingReturnValue, FuncBindingReturnValueError, FuncDescription, FuncDescriptionContents,
    FuncError, SchemaId, SchemaVariantId, StandardModel, StandardModelError,
};
use crate::{Component, ComponentId};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum ConfirmationViewError {
    #[error("ActionPrototypeError: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("ComponentError: {0}")]
    Component(#[from] ComponentError),
    #[error("FixResolverError: {0}")]
    FixResolver(#[from] FixResolverError),
    #[error("FuncError: {0}")]
    Func(#[from] FuncError),
    #[error("FuncBindingReturnValueError: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("StandardModelError: {0}")]
    StandardModel(#[from] StandardModelError),
}

/// A [`ConfirmationView`] is the view of a conceptual "confirmation" corresponding to a child
/// [`AttributeValue`](crate::AttributeValue) of a [`Component`]'s "/root/confirmation" map.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationView {
    /// The child [`AttributeValue`](crate::AttributeValue) of a [`Component`]'s
    /// "/root/confirmation" map.
    ///
    /// The [`Func`](crate::Func) corresponding to the [`value`](crate::AttributeValue)'s
    /// [`AttributePrototype`](crate::AttributePrototype) has a
    /// [`FuncBackendResponseType`](crate::FuncBackendResponseType) of kind
    /// [`Confirmation`](crate::FuncBackendResponseType::Confirmation).
    pub attribute_value_id: AttributeValueId,
    /// The displayed title of the "confirmation".
    pub title: String,
    /// The displayed description of the "confirmation".
    description: Option<String>,

    /// Indicates the [`Schema`](crate::Schema) that the "confirmation" belongs to.
    pub schema_id: SchemaId,
    /// Indicates the [`SchemaVariant`](crate::SchemaVariant) that the "confirmation" belongs to.
    schema_variant_id: SchemaVariantId,
    /// Indicates the [`Component`](crate::Component) that the "confirmation" belongs to.
    pub component_id: ComponentId,

    /// The name of the [`Schema`](crate::Schema).
    schema_name: String,
    /// The name of the [`Component`](crate::Component).
    component_name: String,
    /// Indicates what "group" the [`Schema`](crate::Schema) belongs to. This is purely cosmetic.
    pub provider: Option<String>,

    /// The resulting output of the last execution of the "confirmation" [`Func`](crate::Func).
    output: Option<Vec<String>>,
    /// The overall status of the "confirmation".
    pub status: ConfirmationStatus,
}

impl ConfirmationView {
    /// Assembles [`ConfirmationViews`](ConfirmationView) and
    /// [`RecommendationViews`](RecommendationView) based on the current status of the
    /// "/root/confirmation" [`Prop`](crate::Prop) tree for a given [`Component`](Component).
    ///
    /// # Errors
    ///
    /// Returns [`ConfirmationViewError`] if the either of the "views" could not be assembled.
    #[allow(clippy::too_many_arguments)]
    pub async fn assemble_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
        all_confirmations: &HashMap<String, ConfirmationEntry>,
        cache: &ConfirmationMetadataCache,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        running_fixes: &[Fix],
        schema_name: String,
    ) -> Result<(Vec<Self>, Vec<RecommendationView>), ConfirmationViewError> {
        let mut confirmations = Vec::new();
        let mut recommendations = Vec::new();

        for (confirmation_name, entry) in all_confirmations {
            let (confirmation, recommendations_for_confirmation) =
                Self::assemble_individual_for_component(
                    ctx,
                    component_id,
                    confirmation_name.clone(),
                    entry,
                    cache,
                    schema_id,
                    schema_variant_id,
                    running_fixes,
                    schema_name.clone(),
                )
                .await?;

            if let Some(confirmation) = confirmation {
                confirmations.push(confirmation);
                recommendations.extend(recommendations_for_confirmation);
            }
        }

        Ok((confirmations, recommendations))
    }

    /// This _private_ method assembles an individual [`ConfirmationView`] with
    /// [`RecommendationViews`](RecommendationView) based on an individual [`ConfirmationEntry`] for
    /// a given [`Component`](crate::Component). It should only be called by
    /// [`Self::assemble_for_component`].
    #[allow(clippy::too_many_arguments)]
    async fn assemble_individual_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
        confirmation_name: String,
        entry: &ConfirmationEntry,
        cache: &ConfirmationMetadataCache,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        running_fixes: &[Fix],
        schema_name: String,
    ) -> Result<(Option<Self>, Vec<RecommendationView>), ConfirmationViewError> {
        let (found_func_binding_return_value_id, found_attribute_value_id, found_func_id) =
            match cache.get(&confirmation_name) {
                Some(cache_result) => cache_result,
                None => {
                    warn!(
                        "No confirmation result found for {} on component {}",
                        &confirmation_name, component_id
                    );
                    return Ok((None, vec![]));
                }
            };

        // Collect the output from the func binding return value.
        let mut output = Vec::new();
        if let Some(func_binding_return_value) =
            FuncBindingReturnValue::get_by_id(ctx, found_func_binding_return_value_id).await?
        {
            if let Some(output_streams) = func_binding_return_value.get_output_stream(ctx).await? {
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
                        ConfirmationStatus::Success => (success_description, Some(name), provider),
                        ConfirmationStatus::Failure => (failure_description, Some(name), provider),
                        _ => (None, Some(name), provider),
                    },
                },
                None => (None, None, None),
            };

        let maybe_fix_resolver =
            FixResolver::find_for_confirmation_attribute_value(ctx, *found_attribute_value_id)
                .await?;

        // Gather all the action prototypes from the recommended actions raw strings.
        let mut recommendations = Vec::new();

        for action in &entry.recommended_actions {
            let context = ActionPrototypeContext { schema_variant_id };

            let action_prototype = match ActionPrototype::find_for_context_and_kind(
                ctx, *action, context,
            )
            .await?
            .pop()
            {
                Some(action_prototype) => action_prototype,
                None => {
                    warn!("Confirmation {} recomended {} but no action of that kind could be found for schema variant {}", &confirmation_name, action.as_ref(), schema_variant_id);
                    continue;
                }
            };

            // Find the last fix ran. If a fix has never been ran before for this
            // recommendation (i.e. no fix resolver), that is fine!
            let maybe_last_fix: Option<FixHistoryView> = match maybe_fix_resolver.as_ref() {
                Some(fix_resolver) => {
                    let fix = Fix::get_by_id(ctx, &fix_resolver.last_fix_id())
                        .await?
                        .ok_or_else(|| ComponentError::FixNotFound(fix_resolver.last_fix_id()))?;

                    // Refresh running fixes.
                    // FIXME(paulo,fletcher,nick,paul): hardcoding 5 minutes timeout to avoid permanently fix results
                    if Utc::now().signed_duration_since(fix.timestamp().created_at)
                        > chrono::Duration::minutes(5)
                    {
                        None
                    } else {
                        fix.history_view(ctx, false)
                            .await
                            .map_err(|e| ComponentError::Fix(Box::new(e)))?
                    }
                }
                None => None,
            };

            let recommendation_action_kind = action_prototype.kind().to_owned();

            let action_func = Func::get_by_id(ctx, &action_prototype.func_id())
                .await?
                .ok_or(ActionPrototypeError::FuncNotFound(
                    action_prototype.func_id(),
                    *action_prototype.id(),
                ))?;

            let recommendation_name = action_func.display_name().unwrap_or(action_func.name());

            recommendations.push(RecommendationView {
                confirmation_attribute_value_id: *found_attribute_value_id,
                component_id,
                component_name: Component::find_name(ctx, component_id).await?,
                provider: maybe_provider.clone(),
                name: recommendation_name.to_owned(),
                action_kind: recommendation_action_kind,
                action_prototype_id: *action_prototype.id(),
                last_fix: maybe_last_fix,
                has_running_fix: running_fixes.iter().any(|r| {
                    *r.component_id() == component_id
                        && r.action_prototype_id() == action_prototype.id()
                }),
            });
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
            status: confirmation_status,
            provider: maybe_provider,
        };

        Ok((Some(view), recommendations))
    }
}

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfirmationStatus {
    Failure,
    // FIXME(nick,paulo,paul,wendy): probably remove this once the fix flow is working again.
    NeverStarted,
    Success,
}

/// A [`RecommendationView`] is assembled the result of a "confirmation" and enables users to run
/// [`Fix(es)`](crate::Fix) for a given [`Component`](crate::Component).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationView {
    /// The child [`AttributeValue`](crate::AttributeValue) of a [`Component`]'s
    /// "/root/confirmation" map.
    ///
    /// The [`Func`](crate::Func) corresponding to the [`value`](crate::AttributeValue)'s
    /// [`AttributePrototype`](crate::AttributePrototype) has a
    /// [`FuncBackendResponseType`](crate::FuncBackendResponseType) of kind
    /// [`Confirmation`](crate::FuncBackendResponseType::Confirmation).
    pub confirmation_attribute_value_id: AttributeValueId,
    /// Indicates the [`Component`](crate::Component) that the "confirmation" belongs to.
    pub component_id: ComponentId,
    component_name: String,
    provider: Option<String>,

    /// The title of a [recommendation](Self). An example: "Create EC2 Instance".
    pub name: String,
    /// The [`kind`](crate::action_prototype::ActionKind) of
    /// [`ActionPrototype`](crate::ActionPrototype) that the recommended
    /// [action](crate::ActionPrototype) corresponds to.
    pub action_kind: ActionKind,

    /// The [`ActionPrototype`](crate::ActionPrototype) that the recommendation suggests we run
    /// (the function to run for this fix)
    pub action_prototype_id: ActionPrototypeId,

    /// Indicates if an associated [`Fix`](crate::Fix) is in-flight for the [`RecommendationView`].
    /// The running [`Fix`] may also be (or will become) the "last_fix".
    pub has_running_fix: bool,
    /// Gives the [`history view`](crate::fix::FixHistoryView) of the last [`Fix`](crate::Fix) ran.
    /// This will be empty if the [`RecommendationView`] had never had a corresponding
    /// [`Fix`](crate::Fix) ran before.
    pub last_fix: Option<FixHistoryView>,
}

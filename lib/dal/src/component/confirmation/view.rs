//! This module contains [`ConfirmationView`] and everything related to it (including the ability
//! to assemble [`views`](ConfirmationView).

#![warn(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;

use crate::action_prototype::ActionKind;
use crate::component::confirmation::{ConfirmationEntry, ConfirmationMetadataCache};
use crate::fix::FixHistoryView;
use crate::{
    ActionPrototype, ActionPrototypeError, AttributeValueId, ComponentError, DalContext, Fix,
    FixResolver, FixResolverError, FuncBindingReturnValue, FuncBindingReturnValueError,
    FuncDescription, FuncDescriptionContents, FuncError, SchemaId, SchemaVariantId, StandardModel,
    StandardModelError,
};
use crate::{Component, ComponentId};

#[allow(missing_docs)]
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

    #[error("found conflicting recommendations for component: {0}")]
    FoundConflictingRecommendationsForComponent(ComponentId),
    #[error("found conflicting recommendations in confirmation view: {0:?}")]
    FoundConflictingRecommendationsInConfirmationView(Box<ConfirmationView>),
    #[error("could not find primary action kind for confirmation views: {0:?}")]
    CouldNotFindPrimaryActionKindForConfirmationViews(Vec<ConfirmationView>),
    #[error("could not find primary action kind for recommendations: {0:?}")]
    CouldNotFindPrimaryActionKindForRecommendations(Vec<Recommendation>),
}

/// Tracks the primary [`ActionKind`](ActionKind) if the [`ConfirmationView`] or
/// [`Vec<ConfirmationView>`] has at least one [`Recommendation`]. Otherwise, it tracks that there
/// is _no_ [`Recommendation`](Recommendation).
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PrimaryActionKind {
    /// Indicates that the [`ConfirmationView`] or group of [`ConfirmationViews`](ConfirmationView)
    /// has at least one [`Recommendation`] and what the primary [`ActionKind`] is for it/them.
    HasRecommendations(ActionKind),
    /// Indicates that the [`ConfirmationView`] or group of [`ConfirmationViews`](ConfirmationView)
    /// does not have any [`Recommendations`](Recommendation), which means there cannot be
    /// a primary [`ActionKind`].
    NoRecommendations,
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
    /// A list of [`Recommendations`](Recommendation) that can be used to run [`Fixes`](crate::Fix).
    pub recommendations: Vec<Recommendation>,
}

impl ConfirmationView {
    /// Assembles [`ConfirmationViews`](ConfirmationView) based on the current status of the
    /// "/root/confirmation" [`Prop`](crate::Prop) tree for a given [`Component`](crate::Component).
    ///
    /// # Errors
    ///
    /// Returns [`ConfirmationViewError`] if the [`views`](ConfirmationView) could not be assembled.
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
    ) -> Result<(Vec<Self>, PrimaryActionKind), ConfirmationViewError> {
        let mut results = Vec::new();
        let mut primary_action_kind_tracker: Option<PrimaryActionKind> = None;

        for (confirmation_name, entry) in all_confirmations {
            let (view, individual_primary_action_kind) = Self::assemble_individual_for_component(
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

            // Ensure that the action kind is the same for all confirmation views only if there is
            // at least one recommendation. If the tracker is unset, we will set it.
            match primary_action_kind_tracker {
                Some(initialized_primary_action_kind_tracker) => {
                    match (
                        initialized_primary_action_kind_tracker,
                        individual_primary_action_kind,
                    ) {
                        (
                            PrimaryActionKind::HasRecommendations(all_action_kind),
                            PrimaryActionKind::HasRecommendations(individual_action_kind),
                        ) => {
                            // Only check if we have action kind mismatch if there is at least one
                            // recommendation.
                            if all_action_kind != individual_action_kind {
                                return Err(
                                    ConfirmationViewError::FoundConflictingRecommendationsInConfirmationView(Box::new(view)),
                                );
                            }
                        }
                        (
                            PrimaryActionKind::NoRecommendations,
                            PrimaryActionKind::HasRecommendations(_),
                        ) => {
                            // If our tracker was originally set to "NoRecommendations", then we
                            // will need to override it with the first "ActionKind" found.
                            primary_action_kind_tracker = Some(individual_primary_action_kind)
                        }
                        _ => {}
                    }
                }
                None => {
                    // Initialize the tracker with the first primary action kind seen.
                    primary_action_kind_tracker = Some(individual_primary_action_kind)
                }
            }

            results.push(view);
        }

        let all_primary_action_kind = match primary_action_kind_tracker {
            Some(kind) => kind,
            None if all_confirmations.is_empty() => PrimaryActionKind::NoRecommendations,
            None => {
                return Err(
                    ConfirmationViewError::CouldNotFindPrimaryActionKindForConfirmationViews(
                        results.clone(),
                    ),
                );
            }
        };

        Ok((results, all_primary_action_kind))
    }

    /// This _private_ method assembles an individual [`ConfirmationView`] based on an individual
    /// [`ConfirmationEntry`] for a given [`Component`](crate::Component). It should only be called
    /// by [`Self::assemble_for_component`].
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
    ) -> Result<(Self, PrimaryActionKind), ConfirmationViewError> {
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

        // Gather all the action prototypes from the recommended actions raw strings. We'll also
        // keep track of the primary action kind, which will be "None" if there are no
        // recommendations.
        let mut recommendations = Vec::new();
        let mut maybe_primary_action_kind: Option<ActionKind> = None;

        for action in &entry.recommended_actions {
            let action_prototype =
                ActionPrototype::find_by_name(ctx, action, schema_id, schema_variant_id)
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
                match maybe_fix_resolver.as_ref().and_then(FixResolver::success) {
                    Some(true) => RecommendationStatus::Success,
                    Some(false) => RecommendationStatus::Failure,
                    None => RecommendationStatus::Unstarted,
                }
            };

            // Find the last fix ran. If a fix has never been ran before for this
            // recommendation (i.e. no fix resolver), that is fine!
            let maybe_last_fix: Option<FixHistoryView> = match maybe_fix_resolver.as_ref() {
                Some(fix_resolver) => {
                    let fix = Fix::get_by_id(ctx, &fix_resolver.last_fix_id())
                        .await?
                        .ok_or_else(|| ComponentError::FixNotFound(fix_resolver.last_fix_id()))?;
                    fix.history_view(ctx, false)
                        .await
                        .map_err(|e| ComponentError::Fix(Box::new(e)))?
                }
                None => None,
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
            let recommendation_action_kind = action_prototype.kind().to_owned();

            // Ensure that the action kind is the same for all recommendations. If it is unset, we
            // will set it.
            if let Some(kind) = maybe_primary_action_kind {
                if kind != recommendation_action_kind {
                    return Err(
                        ConfirmationViewError::FoundConflictingRecommendationsForComponent(
                            component_id,
                        ),
                    );
                }
            } else {
                maybe_primary_action_kind = Some(recommendation_action_kind);
            }

            recommendations.push(Recommendation {
                confirmation_attribute_value_id: *found_attribute_value_id,
                component_id,
                component_name: Component::find_name(ctx, component_id).await?,
                provider: maybe_provider.clone(),
                name: workflow_prototype.title().to_owned(),
                recommended_action: action_prototype.name().to_owned(),
                action_kind: recommendation_action_kind,
                status: recommendation_status,
                last_fix: maybe_last_fix,
                is_runnable: recommendation_is_runnable,
            });
        }

        // Find the primary action kind.
        let primary_action_kind = match maybe_primary_action_kind {
            Some(kind) => PrimaryActionKind::HasRecommendations(kind),
            None if recommendations.is_empty() => PrimaryActionKind::NoRecommendations,
            None => {
                return Err(
                    ConfirmationViewError::CouldNotFindPrimaryActionKindForRecommendations(
                        recommendations.clone(),
                    ),
                );
            }
        };

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

        Ok((view, primary_action_kind))
    }
}

#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfirmationStatus {
    Running,
    Failure,
    Success,
    // FIXME(nick,paulo,paul,wendy): probably remove this once the fix flow is working again.
    NeverStarted,
}

// TODO(nick,paulo,paul,wendy): yes, the fields for the "Recommendation" struct are technically
// already on the confirmation itself and we could just map the recommendations back to the
// confirmations in the frontend, but we want shit to work again before optimizing. Fix the fix
// flow!

/// A [`Recommendation`] is assembled the result of a "confirmation" and enables users to run
/// [`Fix(es)`](crate::Fix) for a given [`Component`](crate::Component).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
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
    name: String,
    /// Maps to the name of an [`ActionPrototype`](crate::ActionPrototype). An example: "create".
    pub recommended_action: String,
    /// The [`kind`](crate::action_prototype::ActionKind) of
    /// [`ActionPrototype`](crate::ActionPrototype) that the recommended
    /// [action](crate::ActionPrototype) corresponds to.
    pub action_kind: ActionKind,
    /// The last recorded [`status`](RecommendationStatus) of the [recommendation](Self).
    pub status: RecommendationStatus,
    /// Indicates the ability to "run" the [`Fix`](crate::Fix) associated with the
    /// [recommendation](Self).
    pub is_runnable: RecommendationIsRunnable,
    /// Gives the [`history view`](crate::fix::FixHistoryView) of the last [`Fix`](crate::Fix) ran.
    /// This will be empty if the [`Recommendation`] had never had a corresponding
    /// [`Fix`](crate::Fix) ran before.
    pub last_fix: Option<FixHistoryView>,
}

/// Tracks the last known status of a [`Recommendation`] (corresponds to the
/// [`FixResolver`](crate::FixResolver)).
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
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

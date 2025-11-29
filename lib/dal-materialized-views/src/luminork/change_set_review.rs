use std::collections::HashMap;

use dal::{
    Component,
    DalContext,
};
use si_frontend_mv_types::{
    action::action_diff_list::{
        ActionDiffList,
        ActionDiffView,
    },
    component::{
        ComponentDiffStatus,
        component_diff::AttributeDiff,
    },
    luminork_change_set_review::{
        AttributeDiffTree,
        ComponentReview,
        LuminorkChangeSetReview,
    },
};
use si_id::ComponentId;
use telemetry::prelude::*;

/// Assembles a comprehensive change set review optimized for Luminork.
///
/// This combines data from ComponentList, ComponentDiff, ActionDiffList, and ErasedComponents,
/// and applies frontend processing logic:
/// - Converts flat attribute diffs to tree structure
/// - Filters out uninteresting diffs
/// - Corrects diff status based on meaningful changes and action diffs
/// - Orders components by Added > Modified > Removed
#[instrument(
    name = "dal_materialized_views.luminork.change_set_review",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> crate::Result<LuminorkChangeSetReview> {
    let ctx = &ctx;
    let old_ctx = ctx.clone_with_head().await?;
    let old_ctx = &old_ctx;

    let workspace_pk = ctx.workspace_pk()?;

    // If we're on HEAD, return empty review
    if ctx.change_set_id() == old_ctx.change_set_id() {
        return Ok(LuminorkChangeSetReview {
            id: workspace_pk,
            components: vec![],
        });
    }

    // Get all component IDs in this change set
    let mut component_ids = Component::list_ids(ctx).await?;
    component_ids.sort();

    // Get action diff list
    let action_diff_list = crate::action::action_diff_list::assemble(ctx.clone()).await?;
    let action_diffs_by_component = group_action_diffs_by_component(&action_diff_list);

    // Get erased components (removed from HEAD)
    let erased_components = crate::component::erased_components::assemble(ctx.clone()).await?;

    let mut components = Vec::new();

    // Process current components
    for &component_id in &component_ids {
        let component_in_list =
            crate::component::assemble_in_list(ctx.clone(), component_id).await?;
        let component_diff =
            crate::component::component_diff::assemble(ctx.clone(), component_id).await?;

        let action_diffs = action_diffs_by_component
            .get(&component_id)
            .cloned()
            .unwrap_or_default();

        // Build attribute diff trees and apply filtering
        let attribute_diff_trees = to_attribute_diff_trees(&component_diff.attribute_diffs);

        // Correct the diff status
        let corrected_diff_status = correct_diff_status(
            component_diff.diff_status,
            &attribute_diff_trees,
            &action_diffs,
            component_in_list.to_delete,
        );

        // Only include components with meaningful changes
        if corrected_diff_status != ComponentDiffStatus::None {
            components.push(ComponentReview {
                component: component_in_list,
                attribute_diff_trees,
                action_diffs,
                corrected_diff_status,
            });
        }
    }

    // Add erased components
    for (component_id, erased) in erased_components.erased {
        let action_diffs = action_diffs_by_component
            .get(&component_id)
            .cloned()
            .unwrap_or_default();

        let attribute_diff_trees = to_attribute_diff_trees(&erased.diff.attribute_diffs);

        components.push(ComponentReview {
            component: erased.component,
            attribute_diff_trees,
            action_diffs,
            corrected_diff_status: ComponentDiffStatus::Removed,
        });
    }

    // Sort by diff status: Added -> Modified -> Removed
    components.sort_by_key(|c| match c.corrected_diff_status {
        ComponentDiffStatus::Added => 0,
        ComponentDiffStatus::Modified => 1,
        ComponentDiffStatus::Removed => 2,
        ComponentDiffStatus::None => 3,
    });

    Ok(LuminorkChangeSetReview {
        id: workspace_pk,
        components,
    })
}

/// Groups action diffs by component ID
fn group_action_diffs_by_component(
    action_diff_list: &ActionDiffList,
) -> HashMap<ComponentId, Vec<ActionDiffView>> {
    let mut result: HashMap<ComponentId, Vec<ActionDiffView>> = HashMap::new();

    for action_diff in action_diff_list.action_diffs.values() {
        if action_diff.diff_status
            != si_frontend_mv_types::action::action_diff_list::ActionDiffStatus::None
        {
            result
                .entry(action_diff.component_id)
                .or_default()
                .push(action_diff.clone());
        }
    }

    result
}

/// Converts flat attribute diffs to a filtered, flattened list
fn to_attribute_diff_trees(attribute_diffs: &[(String, AttributeDiff)]) -> Vec<AttributeDiffTree> {
    let mut result = Vec::new();

    for (path, diff) in attribute_diffs {
        // Filter out uninteresting diffs
        if !should_include_diff(path, diff) {
            continue;
        }

        result.push(AttributeDiffTree {
            path: path.clone(),
            diff: diff.clone(),
        });
    }

    // Sort by path for consistent ordering
    result.sort_by(|a, b| a.path.cmp(&b.path));

    result
}

/// Filters out uninteresting diffs (matching frontend logic)
fn should_include_diff(path: &str, diff: &AttributeDiff) -> bool {
    use si_frontend_mv_types::component::component_diff::AttributeSourceAndValue;

    // Filter out internal SI attributes that aren't useful for review
    if path == "/si/type" || path == "/si/color" {
        return false;
    }

    let (old, new): (
        Option<&AttributeSourceAndValue>,
        Option<&AttributeSourceAndValue>,
    ) = match diff {
        AttributeDiff::Added { new } => (None, Some(new)),
        AttributeDiff::Removed { old } => (Some(old), None),
        AttributeDiff::Modified { old, new } => (Some(old), Some(new)),
    };

    // If old and new are identical, skip (can happen on upgrades)
    if let (Some(old_val), Some(new_val)) = (old, new) {
        if old_val == new_val {
            return false;
        }
    }

    // For added/removed values, filter out uninteresting defaults
    if old.is_none() || new.is_none() {
        let Some(source_and_value) = old.or(new) else {
            // Both old and new are None, skip
            return false;
        };

        // Don't show uninteresting default values from schema
        if source_and_value.source.from_schema.unwrap_or(false) {
            if let Some(value) = &source_and_value.value {
                // Filter empty objects/arrays, null, empty string, zero
                if value.is_null()
                    || (value.is_object() && value.as_object().is_some_and(|o| o.is_empty()))
                    || (value.is_array() && value.as_array().is_some_and(|a| a.is_empty()))
                    || (value.is_string() && value.as_str().is_some_and(|s| s.is_empty()))
                    || (value.is_number() && value.as_i64() == Some(0))
                {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Don't show new objects if they are fields of an object (top 2 levels only)
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.len() <= 2 {
            if let si_frontend_mv_types::component::component_diff::SimplifiedAttributeSource::Value { value } = &source_and_value.source.simplified_source {
                if value.is_object() {
                    return false;
                }
            }
        }
    }

    true
}

/// Corrects diff status based on meaningful changes and action diffs
fn correct_diff_status(
    original_status: ComponentDiffStatus,
    attribute_diff_trees: &[AttributeDiffTree],
    action_diffs: &[ActionDiffView],
    to_delete: bool,
) -> ComponentDiffStatus {
    let mut status = original_status;

    // If Modified but no meaningful attribute changes, set to None
    if status == ComponentDiffStatus::Modified {
        let has_meaningful_changes = !attribute_diff_trees.is_empty();

        if !has_meaningful_changes {
            status = ComponentDiffStatus::None;
        }
    }

    // If there are action diffs, we're Modified
    if status == ComponentDiffStatus::None && !action_diffs.is_empty() {
        status = ComponentDiffStatus::Modified;
    }

    // Handle toDelete + Removed case
    if to_delete && original_status == ComponentDiffStatus::Removed {
        status = ComponentDiffStatus::Removed;
    }

    status
}

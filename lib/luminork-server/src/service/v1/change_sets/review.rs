use std::collections::{
    HashMap,
    HashSet,
};

use axum::{
    extract::Query,
    http::StatusCode,
    response::{
        IntoResponse,
        Json,
    },
};
use dal::{
    Component,
    ComponentId,
};
use sdf_extract::{
    EddaClient,
    FriggStore,
};
use serde::Serialize;
use serde_json::Value;
use si_frontend_mv_types::{
    component::component_diff::ComponentDiff,
    reference::ReferenceKind,
};
use telemetry::prelude::*;
use telemetry_utils::monotonic;
use utoipa::ToSchema;

use super::{
    ChangeSetError,
    ChangeSetResult,
};
use crate::{
    extract::change_set::ChangeSetDalContext,
    service::v1::schemas::BuildingResponseV1,
};

/// Query parameters for review endpoint
#[derive(serde::Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReviewQueryParams {
    /// Include resource diff (CloudFormation/Terraform code diffs)
    #[serde(default)]
    pub include_resource_diff: bool,
}

/// Response envelope for change set review endpoint
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ReviewResponseV1 {
    Success(ChangeSetReviewV1Response),
    Building(BuildingResponseV1),
}

impl IntoResponse for ReviewResponseV1 {
    fn into_response(self) -> axum::response::Response {
        match self {
            ReviewResponseV1::Success(response) => (StatusCode::OK, Json(response)).into_response(),
            ReviewResponseV1::Building(response) => {
                (StatusCode::ACCEPTED, Json(response)).into_response()
            }
        }
    }
}

/// Response for change set review endpoint
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetReviewV1Response {
    /// List of components with changes
    pub components: Vec<ComponentReviewV1>,
    /// Summary statistics
    pub summary: ReviewSummaryV1,
}

/// A single component's review data
#[derive(Serialize, Debug, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentReviewV1 {
    /// The component ID
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    /// The component name
    pub component_name: String,
    /// The schema name
    pub schema_name: String,
    /// The diff status (Added, Modified, Removed, None)
    #[schema(value_type = String, example = "Added")]
    pub diff_status: String,
    /// Simplified attribute diffs - easier to consume than raw MV format
    #[schema(value_type = Object)]
    pub attribute_diffs: HashMap<String, SimplifiedAttributeDiffV1>,
    /// Resource diff (code/template diff) - only included if includeResourceDiff=true
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub resource_diff: Option<Value>,
}

/// Simplified attribute diff for easier CLI consumption
#[derive(Serialize, Debug, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SimplifiedAttributeDiffV1 {
    /// The type of change: "added", "removed", "modified"
    pub change_type: String,
    /// The new value (if added or modified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<Value>,
    /// The old value (if removed or modified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<Value>,
    /// How the new value is sourced: "value", "subscription", "prototype"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_source_type: Option<String>,
    /// How the old value is sourced
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_source_type: Option<String>,
    /// For subscriptions: the component name being subscribed to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_source_component_name: Option<String>,
    /// For subscriptions: the component ID being subscribed to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_source_component_id: Option<String>,
    /// For subscriptions: the path being subscribed to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_source_path: Option<String>,
    /// For subscriptions (old): the component name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_source_component_name: Option<String>,
    /// For subscriptions (old): the component ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_source_component_id: Option<String>,
    /// For subscriptions (old): the path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_source_path: Option<String>,
    /// For prototypes: the prototype description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_source_prototype: Option<String>,
    /// For prototypes (old): the prototype description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_source_prototype: Option<String>,
    /// Whether the value came from schema default
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_schema: Option<bool>,
}

/// Component lookup information for subscription resolution
#[derive(Serialize, Debug, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentLookupV1 {
    /// Component name
    pub name: String,
    /// Schema name
    pub schema_name: String,
}

/// Summary statistics for the review
#[derive(Serialize, Debug, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSummaryV1 {
    /// Total number of changed components
    pub total_components: usize,
    /// Number of added components
    pub added: usize,
    /// Number of modified components
    pub modified: usize,
    /// Number of removed components
    pub removed: usize,
}

/// Get a comprehensive review of all changes in a change set
///
/// Returns all components with diffs in a single call. Includes component lookup
/// data for resolving subscription sources, allowing the CLI to display full
/// context without additional API calls.
#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/review",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("includeResourceDiff" = Option<bool>, Query, description = "Include resource code diffs (CloudFormation/Terraform)"),
    ),
    summary = "Get a comprehensive review of all changes in a change set",
    description = "Returns all components with diffs in a single call. Includes component lookup data for resolving subscription sources.",
    tag = "change_sets",
    responses(
        (status = 200, description = "Change set review retrieved", body = ChangeSetReviewV1Response),
        (status = 202, description = "Change set review data is being generated, try again later", body = BuildingResponseV1),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Change set not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn review_change_set(
    ChangeSetDalContext(ctx): ChangeSetDalContext,
    Query(params): Query<ReviewQueryParams>,
    frigg: FriggStore,
    edda_client: EddaClient,
) -> ChangeSetResult<ReviewResponseV1> {
    if ctx.is_head().await? {
        return Err(ChangeSetError::HeadDiffNotAvailable);
    }

    // Get all component IDs in the changeset
    let component_ids = Component::list_ids(&ctx).await?;

    let mut components = Vec::new();
    let mut summary = ReviewSummaryV1 {
        total_components: 0,
        added: 0,
        modified: 0,
        removed: 0,
    };
    let mut mvs_missing = false;

    // Build component lookup map as we go (for subscription resolution)
    let mut all_components_lookup: HashMap<ComponentId, ComponentLookupV1> = HashMap::new();

    // Fetch ComponentDiff MV for each component
    for component_id in component_ids {
        // Fetch ComponentDiff MV from Frigg
        let Some(obj) = frigg
            .get_current_workspace_object(
                ctx.workspace_pk()?,
                ctx.change_set_id(),
                &ReferenceKind::ComponentDiff.to_string(),
                &component_id.to_string(),
            )
            .await?
        else {
            mvs_missing = true;
            continue;
        };

        let Ok(diff) = serde_json::from_value::<ComponentDiff>(obj.data) else {
            continue; // Invalid MV data, skip
        };

        // Filter attribute diffs to remove noise (empty defaults, identical values, etc.)
        let filtered_attribute_diffs = filter_attribute_diffs(diff.attribute_diffs);

        // Recalculate diff status after filtering (based on Review.vue logic)
        let mut recalculated_diff_status = diff.diff_status;

        // If it was Modified but filtering removed all diffs, set to None
        if matches!(
            recalculated_diff_status,
            si_frontend_mv_types::component::ComponentDiffStatus::Modified
        ) && filtered_attribute_diffs.is_empty()
        {
            recalculated_diff_status = si_frontend_mv_types::component::ComponentDiffStatus::None;
        }

        // Skip components with no changes after filtering
        if matches!(
            recalculated_diff_status,
            si_frontend_mv_types::component::ComponentDiffStatus::None
        ) {
            continue;
        }

        // Get component metadata from the graph
        let component = Component::get_by_id(&ctx, component_id).await?;
        let component_name = component.name(&ctx).await?;
        let schema_name = component.schema(&ctx).await?.name;

        // Populate cache for any referenced components we haven't seen yet
        let mut referenced_ids = HashSet::new();
        for (_, attr_diff) in &filtered_attribute_diffs {
            extract_referenced_components(attr_diff, &mut referenced_ids);
        }

        for ref_id in referenced_ids {
            // Only fetch if not already in cache - using entry API
            if let std::collections::hash_map::Entry::Vacant(e) =
                all_components_lookup.entry(ref_id)
            {
                if let Ok(ref_component) = Component::get_by_id(&ctx, ref_id).await {
                    if let (Ok(ref_name), Ok(ref_schema)) = (
                        ref_component.name(&ctx).await,
                        ref_component.schema(&ctx).await,
                    ) {
                        e.insert(ComponentLookupV1 {
                            name: ref_name,
                            schema_name: ref_schema.name,
                        });
                    }
                }
            }
        }

        // Update summary with recalculated status
        summary.total_components += 1;
        match recalculated_diff_status {
            si_frontend_mv_types::component::ComponentDiffStatus::Added => summary.added += 1,
            si_frontend_mv_types::component::ComponentDiffStatus::Modified => summary.modified += 1,
            si_frontend_mv_types::component::ComponentDiffStatus::Removed => summary.removed += 1,
            si_frontend_mv_types::component::ComponentDiffStatus::None => {}
        }

        // Convert to simplified format for CLI
        let simplified_diffs =
            simplify_attribute_diffs(filtered_attribute_diffs, &all_components_lookup);

        components.push(ComponentReviewV1 {
            component_id,
            component_name,
            schema_name,
            diff_status: format!("{recalculated_diff_status:?}"),
            attribute_diffs: simplified_diffs,
            resource_diff: if params.include_resource_diff {
                Some(serde_json::to_value(&diff.resource_diff)?)
            } else {
                None
            },
        });
    }

    // If any MVs were missing, trigger rebuild and return 202
    if mvs_missing {
        if let Err(e) = edda_client
            .rebuild_for_change_set(ctx.workspace_pk()?, ctx.change_set_id())
            .await
        {
            warn!(
                "Failed to send edda rebuild request for change set review {}: {}",
                ctx.change_set_id(),
                e
            );
        }

        monotonic!(luminork_building_change_set_review = 1);
        return Ok(ReviewResponseV1::Building(BuildingResponseV1 {
            status: "building".to_string(),
            message: "Change set review data is being generated, please retry shortly".to_string(),
            retry_after_seconds: 2,
            estimated_completion_seconds: 5,
        }));
    }

    // Sort by status: Added → Modified → Removed for better CLI display
    components.sort_by_key(|c| match c.diff_status.as_str() {
        "Added" => 0,
        "Modified" => 1,
        "Removed" => 2,
        _ => 3,
    });

    Ok(ReviewResponseV1::Success(ChangeSetReviewV1Response {
        components,
        summary,
    }))
}

/// Simplify attribute diffs into a more CLI-friendly format
fn simplify_attribute_diffs(
    attribute_diffs: Vec<(
        String,
        si_frontend_mv_types::component::component_diff::AttributeDiff,
    )>,
    component_lookup: &HashMap<ComponentId, ComponentLookupV1>,
) -> HashMap<String, SimplifiedAttributeDiffV1> {
    use si_frontend_mv_types::component::component_diff::AttributeDiff;

    let mut result = HashMap::new();

    for (path, diff) in attribute_diffs {
        let simplified = match diff {
            AttributeDiff::Added { new } => {
                let (source_type, comp_name, comp_id, source_path, prototype, from_schema) =
                    extract_source_info(&new, component_lookup);

                SimplifiedAttributeDiffV1 {
                    change_type: "added".to_string(),
                    new_value: new.value,
                    old_value: None,
                    new_source_type: Some(source_type),
                    old_source_type: None,
                    new_source_component_name: comp_name,
                    new_source_component_id: comp_id,
                    new_source_path: source_path,
                    old_source_component_name: None,
                    old_source_component_id: None,
                    old_source_path: None,
                    new_source_prototype: prototype,
                    old_source_prototype: None,
                    from_schema,
                }
            }
            AttributeDiff::Removed { old } => {
                let (source_type, comp_name, comp_id, source_path, prototype, from_schema) =
                    extract_source_info(&old, component_lookup);

                SimplifiedAttributeDiffV1 {
                    change_type: "removed".to_string(),
                    new_value: None,
                    old_value: old.value,
                    new_source_type: None,
                    old_source_type: Some(source_type),
                    new_source_component_name: None,
                    new_source_component_id: None,
                    new_source_path: None,
                    old_source_component_name: comp_name,
                    old_source_component_id: comp_id,
                    old_source_path: source_path,
                    new_source_prototype: None,
                    old_source_prototype: prototype,
                    from_schema,
                }
            }
            AttributeDiff::Modified { old, new } => {
                let (
                    new_source_type,
                    new_comp_name,
                    new_comp_id,
                    new_source_path,
                    new_prototype,
                    _,
                ) = extract_source_info(&new, component_lookup);
                let (
                    old_source_type,
                    old_comp_name,
                    old_comp_id,
                    old_source_path,
                    old_prototype,
                    from_schema,
                ) = extract_source_info(&old, component_lookup);

                SimplifiedAttributeDiffV1 {
                    change_type: "modified".to_string(),
                    new_value: new.value,
                    old_value: old.value,
                    new_source_type: Some(new_source_type),
                    old_source_type: Some(old_source_type),
                    new_source_component_name: new_comp_name,
                    new_source_component_id: new_comp_id,
                    new_source_path,
                    old_source_component_name: old_comp_name,
                    old_source_component_id: old_comp_id,
                    old_source_path,
                    new_source_prototype: new_prototype,
                    old_source_prototype: old_prototype,
                    from_schema,
                }
            }
        };

        result.insert(path, simplified);
    }

    result
}

/// Return type for extract_source_info
type SourceInfo = (
    String,         // source_type
    Option<String>, // component_name
    Option<String>, // component_id
    Option<String>, // source_path
    Option<String>, // prototype
    Option<bool>,   // from_schema
);

/// Extract source information from AttributeSourceAndValue
fn extract_source_info(
    source_and_value: &si_frontend_mv_types::component::component_diff::AttributeSourceAndValue,
    component_lookup: &HashMap<ComponentId, ComponentLookupV1>,
) -> SourceInfo {
    use si_frontend_mv_types::component::component_diff::SimplifiedAttributeSource;

    let from_schema = source_and_value.source.from_schema;

    match &source_and_value.source.simplified_source {
        SimplifiedAttributeSource::Value { .. } => {
            ("value".to_string(), None, None, None, None, from_schema)
        }
        SimplifiedAttributeSource::Subscription { component, path } => {
            let comp_name = component_lookup.get(component).map(|c| c.name.clone());

            (
                "subscription".to_string(),
                comp_name,
                Some(component.to_string()),
                Some(path.clone()),
                None,
                from_schema,
            )
        }
        SimplifiedAttributeSource::Prototype { prototype } => (
            "prototype".to_string(),
            None,
            None,
            None,
            Some(prototype.clone()),
            from_schema,
        ),
    }
}

/// Filter attribute diffs to exclude "uninteresting" changes
/// Based on logic from app/web/src/newhotness/Review.vue:shouldIncludeDiff
fn filter_attribute_diffs(
    attribute_diffs: Vec<(
        String,
        si_frontend_mv_types::component::component_diff::AttributeDiff,
    )>,
) -> Vec<(
    String,
    si_frontend_mv_types::component::component_diff::AttributeDiff,
)> {
    attribute_diffs
        .into_iter()
        .filter(|(path, diff)| should_include_diff(path, diff))
        .collect()
}

/// Determine if an attribute diff should be included in the response
fn should_include_diff(
    path: &str,
    diff: &si_frontend_mv_types::component::component_diff::AttributeDiff,
) -> bool {
    use si_frontend_mv_types::component::component_diff::AttributeDiff;

    // Filter out internal SI fields that users don't care about
    if path == "/si/type" || path == "/si/color" {
        return false;
    }

    // If old and new are identical, skip (can happen on schema upgrades)
    if let AttributeDiff::Modified { old, new } = diff {
        if old == new {
            return false;
        }
    }

    // Get the source/value to check
    let source_and_value = match diff {
        AttributeDiff::Added { new } => Some(new),
        AttributeDiff::Removed { old } => Some(old),
        AttributeDiff::Modified { .. } => return true, // Already checked equality above
    };

    if let Some(sv) = source_and_value {
        // Don't show "uninteresting" default values from schema
        if sv.source.from_schema.unwrap_or(false) {
            if let Some(value) = &sv.value {
                // Empty objects/arrays
                if value.is_object() && value.as_object().is_some_and(|o| o.is_empty()) {
                    return false;
                }
                if value.is_array() && value.as_array().is_some_and(|a| a.is_empty()) {
                    return false;
                }
                // Empty string or 0
                if value.is_string() && value.as_str() == Some("") {
                    return false;
                }
                if value.is_number() && value.as_i64() == Some(0) {
                    return false;
                }
                // null
                if value.is_null() {
                    return false;
                }
            } else {
                // value is None (undefined)
                return false;
            }
        }

        // Don't show object field placeholders at top-level (paths like /domain/foo)
        let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if path_segments.len() <= 2 {
            if let si_frontend_mv_types::component::component_diff::SimplifiedAttributeSource::Value {
                value,
            } = &sv.source.simplified_source
            {
                if value.is_object() {
                    return false;
                }
            }
        }
    }

    true
}

/// Extract component IDs from subscription sources in attribute diffs
fn extract_referenced_components(
    attr_diff: &si_frontend_mv_types::component::component_diff::AttributeDiff,
    referenced: &mut HashSet<ComponentId>,
) {
    use si_frontend_mv_types::component::component_diff::{
        AttributeDiff,
        SimplifiedAttributeSource,
    };

    let mut check_source = |source: &SimplifiedAttributeSource| {
        if let SimplifiedAttributeSource::Subscription { component, .. } = source {
            referenced.insert(*component);
        }
    };

    match attr_diff {
        AttributeDiff::Added { new } => check_source(&new.source.simplified_source),
        AttributeDiff::Removed { old } => check_source(&old.source.simplified_source),
        AttributeDiff::Modified { old, new } => {
            check_source(&old.source.simplified_source);
            check_source(&new.source.simplified_source);
        }
    }
}

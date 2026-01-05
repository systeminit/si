use std::{
    collections::HashMap,
    hash::{
        Hash,
        Hasher,
    },
    time::Instant,
};

use axum::{
    extract::Query,
    response::Json,
};
use dal::{
    AttributeValue,
    Component,
    ComponentId,
    action::{
        Action,
        dependency_graph::ActionDependencyGraph,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
    attribute::value::AttributeValueId,
    component::properties::ComponentProperties,
    diagram::Diagram,
};
use serde::Serialize;
use serde_json::{
    Value,
    json,
};
use si_events::ActionState;
use si_id::ManagementPrototypeId;
use telemetry::prelude::*;
use utoipa::{
    self,
    ToSchema,
};

use super::ComponentsError;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::common::QueryStringPaginationParams,
};

#[derive(serde::Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsParams {
    #[serde(flatten)]
    pub pagination: QueryStringPaginationParams,

    // Existing option
    pub include_codegen: Option<bool>,

    // Graph summary options
    pub include_all: Option<bool>,
    pub include_functions: Option<bool>,
    pub include_subscriptions: Option<bool>,
    pub include_manages: Option<bool>,
    pub include_action_functions: Option<bool>,
    pub include_management_functions: Option<bool>,
    pub include_qualification_functions: Option<bool>,
    pub include_resource_info: Option<bool>,
    pub include_diff_status: Option<bool>,
    pub include_execution_history: Option<bool>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsV1Response {
    #[schema(
        value_type = Vec<ComponentDetailsV1>,
        example = json!([
            {
                "component_id": "01H9ZQD35JPMBGHH69BT0Q79AA",
                "name": "my-vpc",
                "schema_name": "AWS::EC2::VPC"
            },
            {
                "component_id": "01H9ZQD35JPMBGHH69BT0Q79BB",
                "name": "Public 1",
                "schema_name": "AWS::EC2::Subnet"
            }
        ])
    )]
    pub component_details: Vec<ComponentDetailsV1>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDetailsV1 {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    pub name: String,
    pub schema_name: String,
    pub codegen: Option<Value>,

    // Optional graph summary fields (included when graph summary parameters are used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_resource: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_status: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<SubscriptionRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub manages: Vec<ManagementRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub action_functions: Vec<FunctionRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub management_functions: Vec<FunctionRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub qualification_functions: Vec<FunctionRelationshipV1>,
}

// Graph summary data structures
#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRelationshipV1 {
    #[schema(value_type = String)]
    pub to_component_id: ComponentId,
    pub to_component_name: String,
    pub from_path: String,
    pub to_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_value: Option<serde_json::Value>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManagementRelationshipV1 {
    #[schema(value_type = String)]
    pub to_component_id: ComponentId,
    pub to_component_name: String,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionRelationshipV1 {
    pub function_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_status: Option<FunctionExecutionStatusV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[schema(value_type = Vec<String>)]
    pub depends_on: Vec<si_id::ActionId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub execution_history: Vec<ExecutionHistoryEntry>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionExecutionStatusV1 {
    pub state: String,
    pub has_active_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub func_run_id: Option<si_id::FuncRunId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub action_id: Option<si_id::ActionId>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionHistoryEntry {
    #[schema(value_type = String)]
    pub func_run_id: si_id::FuncRunId,
    pub state: String,
    #[schema(value_type = String, format = DateTime)]
    pub started_at: chrono::DateTime<chrono::Utc>,
}

// Graph summary implementation - moved from graph module
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSummaryV1 {
    pub component_name: String,
    pub schema_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_resource: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_status: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<SubscriptionRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub manages: Vec<ManagementRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub action_functions: Vec<FunctionRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub management_functions: Vec<FunctionRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub qualification_functions: Vec<FunctionRelationshipV1>,
}

// Helper functions for action execution status
fn get_action_result_from_func_run(func_run: &si_events::FuncRun) -> String {
    if func_run.state() == si_events::FuncRunState::Failure {
        return "Failed".to_string();
    }

    match func_run.action_result_state() {
        Some(si_events::ActionResultState::Success) => "Succeeded".to_string(),
        Some(si_events::ActionResultState::Failure) => "Failed".to_string(),
        Some(si_events::ActionResultState::Unknown) => "Unknown".to_string(),
        None => match func_run.state() {
            si_events::FuncRunState::Success => "Succeeded".to_string(),
            si_events::FuncRunState::Failure => "Failed".to_string(),
            _ => "Idle".to_string(),
        },
    }
}

// Hash-based component diff check
async fn has_meaningful_component_changes(
    ctx: &dal::DalContext,
    component_id: ComponentId,
) -> Result<bool, ComponentsError> {
    use std::collections::hash_map::DefaultHasher;

    let head_ctx = ctx.clone_with_head().await?;
    let exists_in_changeset = Component::exists_by_id(ctx, component_id).await?;
    let exists_in_head = Component::exists_by_id(&head_ctx, component_id).await?;

    if exists_in_changeset != exists_in_head {
        return Ok(true);
    }

    if !exists_in_changeset {
        return Ok(false);
    }

    // Calculate hash of all component attributes in changeset
    let changeset_hash = {
        let mut hasher = DefaultHasher::new();
        if let Some(view) = Component::view_by_id(ctx, component_id).await? {
            if let Ok(props) = ComponentProperties::try_from(view) {
                if let Ok(json_str) = serde_json::to_string(&props) {
                    json_str.hash(&mut hasher);
                }
            }
        }
        if let Ok(action_ids) = Action::find_for_component_id(ctx, component_id).await {
            for action_id in action_ids {
                if let Ok(action) = Action::get_by_id(ctx, action_id).await {
                    action.state().hash(&mut hasher);
                    if let Ok(proto_id) = Action::prototype_id(ctx, action_id).await {
                        proto_id.hash(&mut hasher);
                    }
                }
            }
        }
        hasher.finish()
    };

    // Calculate hash of all component attributes in HEAD
    let head_hash = {
        let mut hasher = DefaultHasher::new();
        if let Some(view) = Component::view_by_id(&head_ctx, component_id).await? {
            if let Ok(props) = ComponentProperties::try_from(view) {
                if let Ok(json_str) = serde_json::to_string(&props) {
                    json_str.hash(&mut hasher);
                }
            }
        }
        if let Ok(action_ids) = Action::find_for_component_id(&head_ctx, component_id).await {
            for action_id in action_ids {
                if let Ok(action) = Action::get_by_id(&head_ctx, action_id).await {
                    action.state().hash(&mut hasher);
                    if let Ok(proto_id) = Action::prototype_id(&head_ctx, action_id).await {
                        proto_id.hash(&mut hasher);
                    }
                }
            }
        }
        hasher.finish()
    };

    Ok(changeset_hash != head_hash)
}

#[allow(clippy::too_many_arguments)]
async fn build_graph_summary(
    ctx: &dal::DalContext,
    include_subscriptions: bool,
    include_manages: bool,
    include_action_functions: bool,
    include_management_functions: bool,
    include_qualification_functions: bool,
    include_resource_info: bool,
    include_diff_status: bool,
    include_execution_history: bool,
) -> Result<std::collections::BTreeMap<String, ComponentSummaryV1>, ComponentsError> {
    let start_time = Instant::now();
    info!("Starting graph summary generation");

    let component_ids = Component::list_ids(ctx).await?;
    debug!("Found {} components for graph summary", component_ids.len());

    let mut component_summaries: std::collections::BTreeMap<String, ComponentSummaryV1> =
        std::collections::BTreeMap::new();

    // Initialize component summaries with basic info
    for component_id in &component_ids {
        let component = Component::get_by_id(ctx, *component_id).await?;
        let component_name = component.name(ctx).await?;
        let schema_name = component.schema(ctx).await?.name;
        let component_id_str = component_id.to_string();

        component_summaries.insert(
            component_id_str,
            ComponentSummaryV1 {
                component_name,
                schema_name,
                has_resource: None,
                resource_id: None,
                resource_status: None,
                has_diff: None,
                diff_status: None,
                subscriptions: Vec::new(),
                manages: Vec::new(),
                action_functions: Vec::new(),
                management_functions: Vec::new(),
                qualification_functions: Vec::new(),
            },
        );
    }

    // Add resource information if requested
    if include_resource_info {
        let resource_start = Instant::now();
        for component_id in &component_ids {
            let component = Component::get_by_id(ctx, *component_id).await?;
            let resource_data = component.resource(ctx).await?;
            let resource_id = component.resource_id(ctx).await?;

            if let Some(summary) = component_summaries.get_mut(&component_id.to_string()) {
                summary.has_resource = Some(resource_data.is_some());
                summary.resource_id = if resource_id.is_empty() {
                    None
                } else {
                    Some(resource_id)
                };
                summary.resource_status = resource_data.map(|r| {
                    let status = r.status;
                    format!("{status:?}")
                });
            }
        }
        debug!(
            "Resource info processed in {}ms",
            resource_start.elapsed().as_millis()
        );
    }

    // Add diff status information if requested
    if include_diff_status {
        let diff_start = Instant::now();
        let head_ctx = ctx.clone_with_head().await?;

        for component_id in &component_ids {
            let exists_in_head = Component::exists_by_id(&head_ctx, *component_id).await?;
            let exists_in_changeset = Component::exists_by_id(ctx, *component_id).await?;

            let (has_diff, diff_status) = if !exists_in_head && exists_in_changeset {
                (true, "Added")
            } else if exists_in_head && !exists_in_changeset {
                (true, "Removed")
            } else if exists_in_head && exists_in_changeset {
                let has_changes = has_meaningful_component_changes(ctx, *component_id).await?;

                if has_changes {
                    debug!(
                        "Component {} ({}) has meaningful changes detected",
                        component_id,
                        Component::get_by_id(ctx, *component_id)
                            .await?
                            .name(ctx)
                            .await?
                    );
                } else {
                    debug!(
                        "Component {} ({}) has no meaningful changes",
                        component_id,
                        Component::get_by_id(ctx, *component_id)
                            .await?
                            .name(ctx)
                            .await?
                    );
                }

                (has_changes, if has_changes { "Modified" } else { "None" })
            } else {
                (false, "None")
            };

            if let Some(summary) = component_summaries.get_mut(&component_id.to_string()) {
                summary.has_diff = Some(has_diff);
                summary.diff_status = Some(diff_status.to_string());
            }
        }
        debug!(
            "Diff status processed in {}ms",
            diff_start.elapsed().as_millis()
        );
    }

    // Add subscription relationships if requested
    if include_subscriptions {
        let diagram_start = Instant::now();
        let diagram = Diagram::assemble(ctx, None).await?;
        debug!(
            "Diagram assembly completed in {}ms",
            diagram_start.elapsed().as_millis()
        );

        for edge in &diagram.attribute_subscription_edges {
            let to_name = Component::name_by_id(ctx, edge.to_component_id).await?;
            let current_value = AttributeValue::view(ctx, edge.to_attribute_value_id).await?;

            let from_component_str = edge.from_component_id.to_string();
            if let Some(summary) = component_summaries.get_mut(&from_component_str) {
                summary.subscriptions.push(SubscriptionRelationshipV1 {
                    to_component_id: edge.to_component_id,
                    to_component_name: to_name,
                    from_path: edge.from_attribute_path.clone(),
                    to_path: edge.to_attribute_path.clone(),
                    current_value,
                });
            }
        }
        debug!("Subscription relationships processed");
    }

    // Add management relationships if requested
    if include_manages {
        for component_id in &component_ids {
            let component = Component::get_by_id(ctx, *component_id).await?;
            let managed_components = component.get_managed(ctx).await?;

            for managed_id in managed_components {
                let to_name = Component::name_by_id(ctx, managed_id).await?;
                let from_component_str = component_id.to_string();

                if let Some(summary) = component_summaries.get_mut(&from_component_str) {
                    summary.manages.push(ManagementRelationshipV1 {
                        to_component_id: managed_id,
                        to_component_name: to_name,
                    });
                }
            }
        }
        debug!("Management relationships processed");
    }

    // Build execution history lookup maps if requested
    let mut action_history_map: HashMap<
        (ComponentId, dal::ActionPrototypeId),
        Vec<ExecutionHistoryEntry>,
    > = HashMap::new();
    let management_history_map: HashMap<
        (ComponentId, ManagementPrototypeId),
        Vec<ExecutionHistoryEntry>,
    > = HashMap::new();
    let mut qualification_history_map: HashMap<
        (ComponentId, AttributeValueId),
        Vec<ExecutionHistoryEntry>,
    > = HashMap::new();

    if include_execution_history {
        let history_start = Instant::now();
        let workspace_pk = ctx.tenancy().workspace_pk().unwrap_or_default();
        info!("Building execution history - include_execution_history is TRUE");

        // Build action execution history using DUAL CONTEXT approach
        if include_action_functions {
            info!("Building action execution history with dual context");

            // Create HEAD context for regular action history (Create/Update/Delete)
            let head_ctx = ctx.clone_with_head().await?;
            let head_workspace_pk = head_ctx.tenancy().workspace_pk().unwrap_or_default();

            // Phase 1: Get regular action func runs from HEAD context (excludes refresh)
            if let Ok(Some(all_func_runs)) = head_ctx
                .layer_db()
                .func_run()
                .read_many_for_workspace(head_workspace_pk)
                .await
            {
                let head_action_runs: Vec<_> = all_func_runs
                    .into_iter()
                    .filter(|fr| fr.function_kind() == si_events::FuncKind::Action)
                    .take(500)
                    .collect();

                info!(
                    "Found {} action func runs from HEAD context",
                    head_action_runs.len()
                );

                for func_run in head_action_runs {
                    if let Some(action_id) = func_run.action_id() {
                        if let (Ok(Some(comp_id)), Ok(proto_id)) = (
                            Action::component_id(&head_ctx, action_id).await,
                            Action::prototype_id(&head_ctx, action_id).await,
                        ) {
                            let result_state = get_action_result_from_func_run(&func_run);

                            let entry = ExecutionHistoryEntry {
                                func_run_id: func_run.id(),
                                state: result_state,
                                started_at: func_run.created_at(),
                            };

                            action_history_map
                                .entry((comp_id, proto_id))
                                .or_default()
                                .push(entry);
                        }
                    }
                }
            }

            // Phase 2: Get refresh action history from CURRENT context (refresh can run on changesets)
            if let Ok(Some(current_func_runs)) = ctx
                .layer_db()
                .func_run()
                .read_many_for_workspace(workspace_pk)
                .await
            {
                let refresh_action_runs: Vec<_> = current_func_runs
                    .into_iter()
                    .filter(|fr| fr.function_kind() == si_events::FuncKind::Action)
                    .take(500)
                    .collect();

                debug!(
                    "Found {} refresh action func runs from current context",
                    refresh_action_runs.len()
                );

                for func_run in refresh_action_runs {
                    if let Some(action_id) = func_run.action_id() {
                        // Check if this is a refresh action by trying current context first
                        if let (Ok(Some(comp_id)), Ok(proto_id)) = (
                            Action::component_id(ctx, action_id).await,
                            Action::prototype_id(ctx, action_id).await,
                        ) {
                            // Get the action prototype to check if it's a refresh action
                            if let Ok(prototype) = ActionPrototype::get_by_id(ctx, proto_id).await {
                                if prototype.kind == ActionKind::Refresh {
                                    let result_state = get_action_result_from_func_run(&func_run);

                                    let entry = ExecutionHistoryEntry {
                                        func_run_id: func_run.id(),
                                        state: result_state,
                                        started_at: func_run.created_at(),
                                    };

                                    action_history_map
                                        .entry((comp_id, proto_id))
                                        .or_default()
                                        .push(entry);
                                }
                            }
                        }
                    }
                }
            }

            debug!(
                "Built complete action history map with {} entries",
                action_history_map.len()
            );
        }

        // Build management execution history (simplified - API changes)
        if include_management_functions {
            debug!("Building management execution history using job states");
            // Simplified - skip complex management history for now
        }

        // Build qualification execution history by parsing actual output logs
        if include_qualification_functions {
            if let Ok(Some(all_func_runs)) = ctx
                .layer_db()
                .func_run()
                .read_many_for_workspace(workspace_pk)
                .await
            {
                let qual_func_runs: Vec<_> = all_func_runs
                    .into_iter()
                    .filter(|fr| fr.function_kind() == si_events::FuncKind::Qualification)
                    .take(500)
                    .collect();

                for func_run in qual_func_runs {
                    if let (Some(comp_id), Some(av_id)) =
                        (func_run.component_id(), func_run.attribute_value_id())
                    {
                        // Parse the actual qualification result from func_run logs
                        let state = if let Ok(Some(func_run_log)) = ctx
                            .layer_db()
                            .func_run_log()
                            .get_for_func_run_id(func_run.id())
                            .await
                        {
                            let func_run_log =
                                std::sync::Arc::<si_events::FuncRunLog>::unwrap_or_clone(
                                    func_run_log,
                                );

                            // Look for output logs that contain the qualification result
                            let mut result_state = None;
                            for log_entry in func_run_log.logs() {
                                if log_entry.stream == "output" {
                                    // Remove "Output: " prefix if present
                                    let json_str = if log_entry.message.starts_with("Output: ") {
                                        &log_entry.message[8..] // Skip "Output: "
                                    } else {
                                        &log_entry.message
                                    };

                                    if let Ok(output_value) =
                                        serde_json::from_str::<serde_json::Value>(json_str)
                                    {
                                        // Check direct status field (new format)
                                        if let Some(status) = output_value.get("status") {
                                            if let Some(status_str) = status.as_str() {
                                                result_state = Some(match status_str {
                                                    "success" => "Succeeded".to_string(),
                                                    "failure" => "Failed".to_string(),
                                                    "warning" => "Warning".to_string(),
                                                    _ => "Completed".to_string(),
                                                });
                                                break;
                                            }
                                        }
                                        // Fallback: check old format (data.result)
                                        else if let Some(data) = output_value.get("data") {
                                            if let Some(result) = data.get("result") {
                                                if let Some(result_str) = result.as_str() {
                                                    result_state = Some(match result_str {
                                                        "success" => "Succeeded".to_string(),
                                                        "failure" => "Failed".to_string(),
                                                        "warning" => "Warning".to_string(),
                                                        _ => "Completed".to_string(),
                                                    });
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Use parsed result or fall back to function execution state
                            result_state.unwrap_or_else(|| match func_run.state() {
                                si_events::FuncRunState::Success => "Succeeded".to_string(),
                                si_events::FuncRunState::Failure => "Failed".to_string(),
                                _ => {
                                    let state = func_run.state();
                                    format!("{state:?}")
                                }
                            })
                        } else {
                            // No logs available, use function execution state
                            match func_run.state() {
                                si_events::FuncRunState::Success => "Succeeded".to_string(),
                                si_events::FuncRunState::Failure => "Failed".to_string(),
                                _ => {
                                    let state = func_run.state();
                                    format!("{state:?}")
                                }
                            }
                        };

                        let entry = ExecutionHistoryEntry {
                            func_run_id: func_run.id(),
                            state,
                            started_at: func_run.created_at(),
                        };

                        qualification_history_map
                            .entry((comp_id, av_id))
                            .or_default()
                            .push(entry);
                    }
                }
            }

            // Sort and limit qualification history
            for history in qualification_history_map.values_mut() {
                history.sort_by(|a, b| b.started_at.cmp(&a.started_at));
                history.truncate(10);
            }
        }

        debug!(
            "Execution history built in {}ms",
            history_start.elapsed().as_millis()
        );
    }

    // Add function relationships based on specific parameters
    let need_any_functions =
        include_action_functions || include_management_functions || include_qualification_functions;
    if need_any_functions {
        let functions_start = Instant::now();

        // Build UNIFIED action lookup: HEAD executions + current changeset scheduled/on-hold
        let mut action_lookup: HashMap<
            (ComponentId, dal::ActionPrototypeId),
            (ActionState, Option<si_id::FuncRunId>, si_id::ActionId),
        > = HashMap::new();

        // Build action dependency graph to get dependencies
        let action_dependency_graph = ActionDependencyGraph::for_workspace(ctx).await?;

        // Phase 1: Get scheduled/on-hold actions from current changeset
        let mut current_changeset_actions = Vec::new();
        for component_id in &component_ids {
            if let Ok(actions) = Action::find_for_component_id(ctx, *component_id).await {
                current_changeset_actions.extend(actions);
            }
        }
        debug!(
            "Found {} actions in current changeset",
            current_changeset_actions.len()
        );

        for action_id in current_changeset_actions {
            if let (Ok(Some(comp_id)), Ok(proto_id), Ok(action)) = (
                Action::component_id(ctx, action_id).await,
                Action::prototype_id(ctx, action_id).await,
                Action::get_by_id(ctx, action_id).await,
            ) {
                let func_run_id = Action::last_func_run_id_for_id_opt(ctx, action_id).await?;

                action_lookup.insert(
                    (comp_id, proto_id),
                    (action.state(), func_run_id, action_id),
                );
            }
        }

        // Phase 2: Get executed actions from HEAD context (ONLY if no current action exists)
        let head_ctx = ctx.clone_with_head().await?;
        let current_change_set_id = ctx.change_set_id();
        let head_change_set_id = head_ctx.change_set_id();

        if current_change_set_id != head_change_set_id {
            let head_actions = Action::list_topologically(&head_ctx).await?;
            debug!("Found {} actions in HEAD context", head_actions.len());

            for action_id in head_actions {
                if let (Ok(Some(comp_id)), Ok(proto_id), Ok(action)) = (
                    Action::component_id(&head_ctx, action_id).await,
                    Action::prototype_id(&head_ctx, action_id).await,
                    Action::get_by_id(&head_ctx, action_id).await,
                ) {
                    // Only add HEAD actions if we don't already have a current action for this component+prototype
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        action_lookup.entry((comp_id, proto_id))
                    {
                        let func_run_id =
                            Action::last_func_run_id_for_id_opt(&head_ctx, action_id).await?;
                        e.insert((action.state(), func_run_id, action_id));
                    }
                }
            }
        }

        info!(
            "Built unified action lookup with {} total entries",
            action_lookup.len()
        );

        // Process each component to add function data
        for component_id in &component_ids {
            if let Some(summary) = component_summaries.get_mut(&component_id.to_string()) {
                let component = Component::get_by_id(ctx, *component_id).await?;

                // Add action functions with REAL status
                if include_action_functions {
                    // Get all action prototypes for this component's schema variant
                    let schema_variant = component.schema_variant(ctx).await?;
                    let schema_variant_id = schema_variant.id();

                    // Get action prototypes from the schema variant
                    if let Ok(action_prototypes) =
                        ActionPrototype::for_variant(ctx, schema_variant_id).await
                    {
                        for prototype in action_prototypes {
                            let key = (*component_id, prototype.id());

                            // Check if there's an actual action scheduled for this function
                            if let Some((state, func_run_id, action_id)) = action_lookup.get(&key) {
                                // There's a real action scheduled
                                let execution_history = if include_execution_history {
                                    action_history_map
                                        .get(&key)
                                        .cloned()
                                        .unwrap_or_else(Vec::new)
                                } else {
                                    Vec::new()
                                };

                                let depends_on: Vec<si_id::ActionId> = action_dependency_graph
                                    .direct_dependencies_of(*action_id)
                                    .into_iter()
                                    .collect();

                                let execution_status = Some(FunctionExecutionStatusV1 {
                                    state: format!("{state:?}"),
                                    has_active_run: *state == ActionState::Running
                                        || *state == ActionState::Dispatched,
                                    func_run_id: *func_run_id,
                                    action_id: Some(*action_id),
                                });

                                summary.action_functions.push(FunctionRelationshipV1 {
                                    function_name: prototype.name.clone(),
                                    execution_status,
                                    depends_on,
                                    execution_history,
                                });
                            } else {
                                // No action scheduled - show as Idle
                                let execution_history = if include_execution_history {
                                    action_history_map
                                        .get(&key)
                                        .cloned()
                                        .unwrap_or_else(Vec::new)
                                } else {
                                    Vec::new()
                                };

                                let execution_status = Some(FunctionExecutionStatusV1 {
                                    state: "Idle".to_string(),
                                    has_active_run: false,
                                    func_run_id: None,
                                    action_id: None,
                                });

                                summary.action_functions.push(FunctionRelationshipV1 {
                                    function_name: prototype.name.clone(),
                                    execution_status,
                                    depends_on: Vec::new(),
                                    execution_history,
                                });
                            }
                        }
                    }
                }

                // Add management functions with execution state
                if include_management_functions {
                    // Get management prototypes from the schema variant
                    let schema_variant = component.schema_variant(ctx).await?;
                    let schema_variant_id = schema_variant.id();

                    if let Ok(mgmt_prototypes) =
                        dal::management::prototype::ManagementPrototype::list_for_variant_id(
                            ctx,
                            schema_variant_id,
                        )
                        .await
                    {
                        for mgmt_prototype in mgmt_prototypes {
                            // Get execution history for this management function
                            let execution_history = if include_execution_history {
                                management_history_map
                                    .get(&(*component_id, mgmt_prototype.id()))
                                    .cloned()
                                    .unwrap_or_else(Vec::new)
                            } else {
                                Vec::new()
                            };

                            // Determine execution status based on most recent job state
                            let execution_status = if !execution_history.is_empty() {
                                let most_recent = &execution_history[0]; // Already sorted by most recent first
                                Some(FunctionExecutionStatusV1 {
                                    state: match most_recent.state.as_str() {
                                        "success" => "Succeeded".to_string(),
                                        "failure" => "Failed".to_string(),
                                        "running" => "Running".to_string(),
                                        _ => most_recent.state.clone(),
                                    },
                                    has_active_run: most_recent.state == "running",
                                    func_run_id: Some(most_recent.func_run_id),
                                    action_id: None,
                                })
                            } else {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Available".to_string(),
                                    has_active_run: false,
                                    func_run_id: None,
                                    action_id: None,
                                })
                            };

                            summary.management_functions.push(FunctionRelationshipV1 {
                                function_name: mgmt_prototype.name.clone(),
                                execution_status,
                                depends_on: Vec::new(), // Management functions don't have action dependencies
                                execution_history,
                            });
                        }
                    }
                }

                // Add qualification functions with REAL status
                if include_qualification_functions {
                    let qualification_avs =
                        Component::list_qualification_avs(ctx, *component_id).await?;

                    for qualification_av in &qualification_avs {
                        if let Ok(Some(qualification)) =
                            dal::qualification::QualificationView::new(ctx, qualification_av.id())
                                .await
                        {
                            let qual_func_run = ctx
                                .layer_db()
                                .func_run()
                                .get_last_qualification_for_attribute_value_id(
                                    ctx.events_tenancy().workspace_pk,
                                    qualification_av.id(),
                                )
                                .await?;

                            let func_run_id = qual_func_run.as_ref().map(|run| run.id());

                            let execution_status = if qualification.finalized {
                                if let Some(result) = &qualification.result {
                                    match result.status {
                                        dal::qualification::QualificationSubCheckStatus::Success => {
                                            Some(FunctionExecutionStatusV1 {
                                                state: "Succeeded".to_string(),
                                                has_active_run: false,
                                                func_run_id,
                                                action_id: None,
                                            })
                                        }
                                        dal::qualification::QualificationSubCheckStatus::Failure => {
                                            Some(FunctionExecutionStatusV1 {
                                                state: "Failed".to_string(),
                                                has_active_run: false,
                                                func_run_id,
                                                action_id: None,
                                            })
                                        }
                                        dal::qualification::QualificationSubCheckStatus::Warning => {
                                            Some(FunctionExecutionStatusV1 {
                                                state: "Warning".to_string(),
                                                has_active_run: false,
                                                func_run_id,
                                                action_id: None,
                                            })
                                        }
                                        _ => {
                                            Some(FunctionExecutionStatusV1 {
                                                state: "Completed".to_string(),
                                                has_active_run: false,
                                                func_run_id,
                                                action_id: None,
                                            })
                                        }
                                    }
                                } else {
                                    Some(FunctionExecutionStatusV1 {
                                        state: "Completed".to_string(),
                                        has_active_run: false,
                                        func_run_id,
                                        action_id: None,
                                    })
                                }
                            } else {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Running".to_string(),
                                    has_active_run: true,
                                    func_run_id,
                                    action_id: None,
                                })
                            };

                            // Get execution history for this qualification function
                            let execution_history = if include_execution_history {
                                qualification_history_map
                                    .get(&(*component_id, qualification_av.id()))
                                    .cloned()
                                    .unwrap_or_else(Vec::new)
                            } else {
                                Vec::new()
                            };

                            summary
                                .qualification_functions
                                .push(FunctionRelationshipV1 {
                                    function_name: qualification.qualification_name,
                                    execution_status,
                                    depends_on: Vec::new(), // Qualification functions don't have action dependencies
                                    execution_history,
                                });
                        }
                    }
                }
            }
        }

        debug!(
            "Function relationships completed in {}ms",
            functions_start.elapsed().as_millis()
        );
    }

    info!(
        "Graph summary completed in {}ms",
        start_time.elapsed().as_millis()
    );
    Ok(component_summaries)
}

// (Hash-based component diff check already defined above)

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("limit" = Option<String>, Query, description = "Maximum number of results to return (default: 50, max: 300)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination (ComponentId of the last item from previous page)"),
        ("includeCodegen" = Option<bool>, Query, description = "Allow returning the codegen for the cloudformation template for the component (if it exists)"),
        ("includeAll" = Option<bool>, Query, description = "Include all graph summary data (equivalent to enabling all include options)"),
        ("includeFunctions" = Option<bool>, Query, description = "Include all function types (action, management, qualification)"),
        ("includeSubscriptions" = Option<bool>, Query, description = "Include subscription relationships"),
        ("includeManages" = Option<bool>, Query, description = "Include management relationships"),
        ("includeActionFunctions" = Option<bool>, Query, description = "Include action function relationships"),
        ("includeManagementFunctions" = Option<bool>, Query, description = "Include management function relationships"),
        ("includeQualificationFunctions" = Option<bool>, Query, description = "Include qualification function relationships"),
        ("includeResourceInfo" = Option<bool>, Query, description = "Include resource information (resource ID and status)"),
        ("includeDiffStatus" = Option<bool>, Query, description = "Include component diff status vs HEAD (Added/Modified/None)"),
        ("includeExecutionHistory" = Option<bool>, Query, description = "Include last 10 execution history entries for each function"),
    ),
    summary = "List all components",
    tag = "components",
    responses(
        (status = 200, description = "Components retrieved successfully", body = ListComponentsV1Response, example = json!({
                    "componentDetails": [
                        {
                            "component_id": "01H9ZQD35JPMBGHH69BT0Q79AA",
                            "name": "my-vpc",
                            "schema_name": "AWS::EC2::VPC"
                        },
                        {
                            "component_id": "01H9ZQD35JPMBGHH69BT0Q79BB",
                            "name": "Public 1",
                            "schema_name": "AWS::EC2::Subnet"
                        }
                    ],
                    "nextCursor": null
                })),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(deprecated)]
pub async fn list_components(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Query(params): Query<ListComponentsParams>,
    tracker: PosthogEventTracker,
) -> Result<Json<ListComponentsV1Response>, ComponentsError> {
    // Set default limit and enforce a max limit
    let limit = params
        .pagination
        .limit
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(50)
        .min(300) as usize;
    let cursor = params.pagination.cursor;

    let mut comp_details = Vec::with_capacity(limit);

    // Get all component
    let mut all_components = Component::list(ctx).await?;

    // Sort components  for consistent pagination
    all_components.sort_by_key(|c| c.id());

    // Find the start index by matching the stringified ComponentId
    let start_index = if let Some(ref cursor_str) = cursor {
        match all_components
            .iter()
            .position(|component| component.id().to_string() == *cursor_str)
        {
            Some(index) => index + 1, // Start after the cursor
            None => 0,
        }
    } else {
        0 // Start from the beginning
    };

    // Compute the end index and extract the paginated slice
    let end_index = (start_index + limit).min(all_components.len());
    let paginated_components: Vec<Component> = all_components[start_index..end_index].to_vec();

    // Generate the next cursor from the last item's ID
    let next_cursor = if end_index < all_components.len() && !paginated_components.is_empty() {
        paginated_components
            .last()
            .map(|component| component.id().to_string())
    } else {
        None
    };

    for component in &paginated_components {
        let name = component.name(ctx).await?;
        let schema_name = component.schema(ctx).await?.name;

        let mut comp_response = ComponentDetailsV1 {
            component_id: component.id(),
            name,
            schema_name,
            codegen: None,

            // Initialize graph summary fields
            has_resource: None,
            resource_id: None,
            resource_status: None,
            has_diff: None,
            diff_status: None,
            subscriptions: Vec::new(),
            manages: Vec::new(),
            action_functions: Vec::new(),
            management_functions: Vec::new(),
            qualification_functions: Vec::new(),
        };

        if let Some(codegen) = params.include_codegen {
            if codegen {
                let code_map_av_id =
                    Component::find_code_map_attribute_value_id(ctx, component.id()).await?;

                let view = AttributeValue::view(ctx, code_map_av_id).await?;
                if let Some(v) = view {
                    let details = v.get("awsCloudFormationLint");
                    comp_response.codegen = details.cloned();
                }
            }
        }

        comp_details.push(comp_response);
    }

    // If any graph summary options are enabled, populate the graph data
    let include_any_graph_data = params.include_all.unwrap_or(false)
        || params.include_functions.unwrap_or(false)
        || params.include_subscriptions.unwrap_or(false)
        || params.include_manages.unwrap_or(false)
        || params.include_action_functions.unwrap_or(false)
        || params.include_management_functions.unwrap_or(false)
        || params.include_qualification_functions.unwrap_or(false)
        || params.include_resource_info.unwrap_or(false)
        || params.include_diff_status.unwrap_or(false)
        || params.include_execution_history.unwrap_or(false);

    if include_any_graph_data {
        // Use the existing graph summary function with the same parameter logic
        let include_all = params.include_all.unwrap_or(false);
        let include_functions = params.include_functions.unwrap_or(false);
        let include_subscriptions = include_all || params.include_subscriptions.unwrap_or(false);
        let include_manages = include_all || params.include_manages.unwrap_or(false);
        let include_action_functions =
            include_all || include_functions || params.include_action_functions.unwrap_or(false);
        let include_management_functions = include_all
            || include_functions
            || params.include_management_functions.unwrap_or(false);
        let include_qualification_functions = include_all
            || include_functions
            || params.include_qualification_functions.unwrap_or(false);
        let include_resource_info = include_all || params.include_resource_info.unwrap_or(false);
        let include_diff_status = include_all || params.include_diff_status.unwrap_or(false);
        let include_execution_history =
            include_all || params.include_execution_history.unwrap_or(false);

        // Build graph summary for the components
        let graph_summaries = build_graph_summary(
            ctx,
            include_subscriptions,
            include_manages,
            include_action_functions,
            include_management_functions,
            include_qualification_functions,
            include_resource_info,
            include_diff_status,
            include_execution_history,
        )
        .await?;

        // Merge graph summary data into component details
        for comp_detail in &mut comp_details {
            if let Some(graph_data) = graph_summaries.get(&comp_detail.component_id.to_string()) {
                comp_detail.has_resource = graph_data.has_resource;
                comp_detail.resource_id = graph_data.resource_id.clone();
                comp_detail.resource_status = graph_data.resource_status.clone();
                comp_detail.has_diff = graph_data.has_diff;
                comp_detail.diff_status = graph_data.diff_status.clone();
                comp_detail.subscriptions = graph_data.subscriptions.clone();
                comp_detail.manages = graph_data.manages.clone();
                comp_detail.action_functions = graph_data.action_functions.clone();
                comp_detail.management_functions = graph_data.management_functions.clone();
                comp_detail.qualification_functions = graph_data.qualification_functions.clone();
            }
        }
    }

    tracker.track(ctx, "api_list_components", json!({}));

    Ok(Json(ListComponentsV1Response {
        component_details: comp_details,
        next_cursor,
    }))
}

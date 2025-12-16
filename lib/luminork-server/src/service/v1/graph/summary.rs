use axum::response::Json;
use std::collections::HashMap;
use dal::{
    Component,
    ComponentId,
    diagram::Diagram,
    attribute::value::{AttributeValue, AttributeValueId},
    action::{Action, dependency_graph::ActionDependencyGraph, prototype::{ActionPrototype, ActionKind}},
    component::properties::ComponentProperties,
};
use si_id::ManagementPrototypeId;
use si_events::ActionState;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{self, ToSchema};
use telemetry::prelude::*;
use std::time::Instant;

use crate::service::v1::components::ComponentsError;

// Simple helper to get action result status directly from func run
// Hash-based component diff check: compares all attributes + action states
async fn has_meaningful_component_changes(
    ctx: &dal::DalContext,
    component_id: ComponentId,
) -> Result<bool, ComponentsError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Create HEAD context for comparison
    let head_ctx = ctx.clone_with_head().await?;
    
    // Check if component exists in both contexts
    let exists_in_changeset = Component::exists_by_id(ctx, component_id).await?;
    let exists_in_head = Component::exists_by_id(&head_ctx, component_id).await?;
    
    // If existence differs, there's definitely a change
    if exists_in_changeset != exists_in_head {
        return Ok(true);
    }
    
    // If component doesn't exist in either, no changes
    if !exists_in_changeset {
        return Ok(false);
    }
    
    // Calculate hash of all component attributes in changeset
    let changeset_hash = {
        let mut hasher = DefaultHasher::new();
        
        // Hash component view (all attributes)
        if let Some(view) = Component::view_by_id(ctx, component_id).await? {
            // Convert to properties and hash the JSON representation
            if let Ok(props) = ComponentProperties::try_from(view) {
                if let Ok(json_str) = serde_json::to_string(&props) {
                    json_str.hash(&mut hasher);
                }
            }
        }
        
        // Hash action states for this component
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
        
        // Hash component view (all attributes)
        if let Some(view) = Component::view_by_id(&head_ctx, component_id).await? {
            // Convert to properties and hash the JSON representation
            if let Ok(props) = ComponentProperties::try_from(view) {
                if let Ok(json_str) = serde_json::to_string(&props) {
                    json_str.hash(&mut hasher);
                }
            }
        }
        
        // Hash action states for this component
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
    
    // Components differ if hashes differ
    Ok(changeset_hash != head_hash)
}

fn get_action_result_from_func_run(func_run: &si_events::FuncRun) -> String {
    // Priority 1: Check if function execution failed
    if func_run.state() == si_events::FuncRunState::Failure {
        return "Failed".to_string();
    }
    
    // Priority 2: Check action result state (the actual AWS operation result)
    match func_run.action_result_state() {
        Some(si_events::ActionResultState::Success) => "Succeeded".to_string(),
        Some(si_events::ActionResultState::Failure) => "Failed".to_string(),
        Some(si_events::ActionResultState::Unknown) => "Unknown".to_string(),
        None => {
            // No action result - check function execution state
            match func_run.state() {
                si_events::FuncRunState::Success => "Succeeded".to_string(),
                si_events::FuncRunState::Failure => "Failed".to_string(),
                _ => "Idle".to_string()
            }
        }
    }
}


use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRelationshipsParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    // Individual inclusion options
    pub include_subscriptions: Option<bool>,
    pub include_manages: Option<bool>,
    pub include_action_functions: Option<bool>,
    pub include_management_functions: Option<bool>,
    pub include_qualification_functions: Option<bool>,
    pub include_resource_info: Option<bool>,
    pub include_diff_status: Option<bool>,
    pub include_execution_history: Option<bool>,
    pub show_only_immediate_dependencies: Option<bool>,
    // Convenience options
    pub include_functions: Option<bool>,  // All function types
    pub include_all: Option<bool>,        // Everything
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRelationshipsV1Response {
    #[schema(value_type = std::collections::BTreeMap<String, ComponentSummaryV1>)]
    pub graph_summary: std::collections::BTreeMap<String, ComponentSummaryV1>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
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
    info!("🚀 [PERF] Starting graph summary generation...");
    
    let mut component_summaries: std::collections::BTreeMap<String, ComponentSummaryV1> = std::collections::BTreeMap::new();
    
    // Get all components
    let component_ids = Component::list_ids(ctx).await?;
    info!("📋 [PERF] Found {} components", component_ids.len());
    
    // Initialize component summaries with basic info
    for component_id in &component_ids {
        let component = Component::get_by_id(ctx, *component_id).await?;
        let component_name = component.name(ctx).await?;
        let schema_name = component.schema(ctx).await?.name;
        let component_id_str = component_id.to_string();
        
        component_summaries.insert(component_id_str, ComponentSummaryV1 {
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
        });
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
                summary.resource_id = if resource_id.is_empty() { None } else { Some(resource_id) };
                summary.resource_status = resource_data.map(|r| format!("{:?}", r.status));
            }
        }
        info!("💾 [PERF] Resource info processed in {}ms", resource_start.elapsed().as_millis());
    }
    
    // Add diff status information if requested
    if include_diff_status {
        let diff_start = Instant::now();
        let head_ctx = ctx.clone_with_head().await?;
        
        for component_id in &component_ids {
            // Check existence in both contexts
            let exists_in_head = Component::exists_by_id(&head_ctx, *component_id).await?;
            let exists_in_changeset = Component::exists_by_id(ctx, *component_id).await?;
            
            let (has_diff, diff_status) = if !exists_in_head && exists_in_changeset {
                (true, "Added")
            } else if exists_in_head && !exists_in_changeset {
                (true, "Removed")
            } else if exists_in_head && exists_in_changeset {
                // Check for meaningful differences (properties, subscriptions, action states)
                let has_changes = has_meaningful_component_changes(ctx, *component_id).await?;
                
                if has_changes {
                    tracing::info!("📊 [DIFF] Component {} ({}) has meaningful changes detected", component_id, 
                        Component::get_by_id(ctx, *component_id).await?.name(ctx).await?);
                } else {
                    tracing::info!("📊 [DIFF] Component {} ({}) has no meaningful changes", component_id,
                        Component::get_by_id(ctx, *component_id).await?.name(ctx).await?);
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
        info!("🔍 [PERF] Diff status processed in {}ms", diff_start.elapsed().as_millis());
    }
    
    // Add subscription relationships if requested
    if include_subscriptions {
        let diagram_start = Instant::now();
        let diagram = Diagram::assemble(ctx, None).await?;
        info!("📊 [PERF] Diagram assembly completed in {}ms", diagram_start.elapsed().as_millis());
        
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
        info!("🔗 [PERF] Subscription relationships processed");
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
        info!("👑 [PERF] Management relationships processed");
    }
    
    // Build execution history lookup maps if requested
    let mut action_history_map: HashMap<(ComponentId, dal::ActionPrototypeId), Vec<ExecutionHistoryEntry>> = HashMap::new();
    let mut management_history_map: HashMap<(ComponentId, ManagementPrototypeId), Vec<ExecutionHistoryEntry>> = HashMap::new();
    let mut qualification_history_map: HashMap<(ComponentId, AttributeValueId), Vec<ExecutionHistoryEntry>> = HashMap::new();
    
    if include_execution_history {
        let history_start = Instant::now();
        let workspace_pk = ctx.tenancy().workspace_pk().unwrap_or_default();
        
        // Build action execution history using DUAL CONTEXT approach
        if include_action_functions {
            info!("📊 [DEBUG] Building action execution history with dual context...");
            
            // Create HEAD context for regular action history (Create/Update/Delete)
            let head_ctx = ctx.clone_with_head().await?;
            let head_workspace_pk = head_ctx.tenancy().workspace_pk().unwrap_or_default();
            
            // Get regular action history from HEAD context
            if let Ok(Some(head_func_runs)) = head_ctx.layer_db().func_run().read_many_for_workspace(head_workspace_pk).await {
                let head_action_runs: Vec<_> = head_func_runs
                    .into_iter()
                    .filter(|fr| fr.function_kind() == si_events::FuncKind::Action)
                    .take(1000)
                    .collect();
                    
                info!("📊 [DEBUG] Found {} action func runs from HEAD context", head_action_runs.len());
                
                for func_run in head_action_runs {
                    if let Some(action_id) = func_run.action_id() {
                        if let (Ok(Some(comp_id)), Ok(proto_id)) = (
                            Action::component_id(&head_ctx, action_id).await,
                            Action::prototype_id(&head_ctx, action_id).await
                        ) {
                            // Get the action result status directly from func run (much simpler!)
                            let result_state = get_action_result_from_func_run(&func_run);
                            
                            let entry = ExecutionHistoryEntry {
                                func_run_id: func_run.id(),
                                state: result_state,
                                started_at: func_run.created_at(),
                            };
                            
                            action_history_map
                                .entry((comp_id, proto_id))
                                .or_insert_with(Vec::new)
                                .push(entry);
                        }
                    }
                }
            }
            
            // Get refresh action history from CURRENT context (refresh can run on changesets)
            if let Ok(Some(current_func_runs)) = ctx.layer_db().func_run().read_many_for_workspace(workspace_pk).await {
                let refresh_action_runs: Vec<_> = current_func_runs
                    .into_iter()
                    .filter(|fr| fr.function_kind() == si_events::FuncKind::Action)
                    .take(500)
                    .collect();
                    
                info!("📊 [DEBUG] Found {} refresh action func runs from current context", refresh_action_runs.len());
                
                for func_run in refresh_action_runs {
                    if let Some(action_id) = func_run.action_id() {
                        // Check if this is a refresh action by trying current context first
                        if let (Ok(Some(comp_id)), Ok(proto_id)) = (
                            Action::component_id(ctx, action_id).await,
                            Action::prototype_id(ctx, action_id).await
                        ) {
                            // Get the action prototype to check if it's a refresh action
                            if let Ok(prototype) = ActionPrototype::get_by_id(ctx, proto_id).await {
                                if prototype.kind == ActionKind::Refresh {
                                    // Use same simple logic for refresh action results
                                    let result_state = get_action_result_from_func_run(&func_run);
                                    
                                    let entry = ExecutionHistoryEntry {
                                        func_run_id: func_run.id(),
                                        state: result_state,
                                        started_at: func_run.created_at(),
                                    };
                                    
                                    // Add refresh action history to current context map
                                    action_history_map
                                        .entry((comp_id, proto_id))
                                        .or_insert_with(Vec::new)
                                        .push(entry);
                                }
                            }
                        }
                    }
                }
            }
            
            // Sort and limit each function's history to last 10
            for history in action_history_map.values_mut() {
                history.sort_by(|a, b| b.started_at.cmp(&a.started_at)); // Newest first
                history.truncate(10);
            }
            info!("📊 [DEBUG] Built complete action history map with {} entries", action_history_map.len());
        }
        
        // Build management execution history (use direct job state lookup approach)
        if include_management_functions {
            info!("📊 [DEBUG] Building management execution history using job states...");
            // Instead of using layer cache func runs, build history directly when processing functions
            // This avoids the complex prototype ID mapping issue
        }
        
        // Build qualification execution history by parsing actual output logs
        if include_qualification_functions {
            if let Ok(Some(all_func_runs)) = ctx.layer_db().func_run().read_many_for_workspace(workspace_pk).await {
                let qual_func_runs: Vec<_> = all_func_runs
                    .into_iter()
                    .filter(|fr| fr.function_kind() == si_events::FuncKind::Qualification)
                    .take(500)
                    .collect();
                    
                for func_run in qual_func_runs {
                    if let (Some(comp_id), Some(av_id)) = (func_run.component_id(), func_run.attribute_value_id()) {
                        // Parse the actual qualification result from func_run logs
                        let state = if let Ok(Some(func_run_log)) = ctx.layer_db().func_run_log().get_for_func_run_id(func_run.id()).await {
                            let func_run_log = std::sync::Arc::<si_events::FuncRunLog>::unwrap_or_clone(func_run_log);
                            
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
                                    
                                    if let Ok(output_value) = serde_json::from_str::<serde_json::Value>(json_str) {
                                        if let Some(data) = output_value.get("data") {
                                            if let Some(result) = data.get("result") {
                                                if let Some(result_str) = result.as_str() {
                                                    result_state = Some(match result_str {
                                                        "success" => "Succeeded".to_string(),
                                                        "failure" => "Failed".to_string(), 
                                                        "warning" => "Warning".to_string(),
                                                        _ => "Completed".to_string()
                                                    });
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Use parsed result or fall back to function execution state
                            result_state.unwrap_or_else(|| {
                                match func_run.state() {
                                    si_events::FuncRunState::Success => "Succeeded".to_string(),
                                    si_events::FuncRunState::Failure => "Failed".to_string(),
                                    _ => format!("{:?}", func_run.state())
                                }
                            })
                        } else {
                            // No logs available, use function execution state
                            match func_run.state() {
                                si_events::FuncRunState::Success => "Succeeded".to_string(),
                                si_events::FuncRunState::Failure => "Failed".to_string(),
                                _ => format!("{:?}", func_run.state())
                            }
                        };

                        let entry = ExecutionHistoryEntry {
                            func_run_id: func_run.id(),
                            state,
                            started_at: func_run.created_at(),
                        };
                        
                        qualification_history_map
                            .entry((comp_id, av_id))
                            .or_insert_with(Vec::new)
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
        
        info!("📊 [PERF] Execution history built in {}ms", history_start.elapsed().as_millis());
    }

    // Add function relationships based on specific parameters
    let need_any_functions = include_action_functions || include_management_functions || include_qualification_functions;
    if need_any_functions {
        let functions_start = Instant::now();
        
        // Build UNIFIED action lookup: HEAD executions + current changeset scheduled/on-hold
        let mut action_lookup: HashMap<(ComponentId, dal::ActionPrototypeId), (ActionState, Option<si_id::FuncRunId>, si_id::ActionId)> = HashMap::new();
        
        // Build action dependency graph to get dependencies
        let action_dependency_graph = ActionDependencyGraph::for_workspace(ctx).await?;
        
        // Phase 1: Get scheduled/on-hold actions from current changeset
        let current_changeset_actions = Action::list_topologically(ctx).await?;
        info!("📊 [DEBUG] Found {} actions in current changeset", current_changeset_actions.len());
        
        for action_id in current_changeset_actions {
            if let (Ok(Some(comp_id)), Ok(proto_id), Ok(action)) = (
                Action::component_id(ctx, action_id).await,
                Action::prototype_id(ctx, action_id).await,
                Action::get_by_id(ctx, action_id).await
            ) {
                let func_run_id = Action::last_func_run_id_for_id_opt(ctx, action_id).await?;
                action_lookup.insert((comp_id, proto_id), (action.state(), func_run_id, action_id));
            }
        }
        
        // Phase 2: Get executed actions from HEAD context (if different from current)
        let head_ctx = ctx.clone_with_head().await?;
        let current_change_set_id = ctx.change_set_id();
        let head_change_set_id = head_ctx.change_set_id();
        
        if current_change_set_id != head_change_set_id {
            let head_actions = Action::list_topologically(&head_ctx).await?;
            info!("📊 [DEBUG] Found {} actions in HEAD context", head_actions.len());
            
            for action_id in head_actions {
                if let (Ok(Some(comp_id)), Ok(proto_id), Ok(action)) = (
                    Action::component_id(&head_ctx, action_id).await,
                    Action::prototype_id(&head_ctx, action_id).await,
                    Action::get_by_id(&head_ctx, action_id).await
                ) {
                    // Only add HEAD actions if we don't already have a current action for this component+prototype
                    if !action_lookup.contains_key(&(comp_id, proto_id)) {
                        let func_run_id = Action::last_func_run_id_for_id_opt(&head_ctx, action_id).await?;
                        action_lookup.insert((comp_id, proto_id), (action.state(), func_run_id, action_id));
                    }
                }
            }
        }
        
        info!("📊 [DEBUG] Built unified action lookup with {} total entries", action_lookup.len());
        
        for component_id in &component_ids {
            let (management_functions, action_functions) = crate::service::v1::components::get_component_functions(ctx, *component_id).await?;
            let component_str = component_id.to_string();
            
            if let Some(summary) = component_summaries.get_mut(&component_str) {
                // Add action functions with real-time states if requested
                if include_action_functions {
                    for action_func in action_functions {
                        let (execution_status, depends_on) = if let Some((state, func_run_id, action_id)) = action_lookup.get(&(*component_id, action_func.prototype_id)) {
                            // Get dependencies for this action
                            let dependencies = action_dependency_graph.direct_dependencies_of(*action_id);
                            
                            // Determine status using combined approach: function execution + resource result
                            let execution_status = match state {
                                ActionState::Running | ActionState::Dispatched => {
                                    // Currently executing - show as running
                                    Some(FunctionExecutionStatusV1 {
                                        state: "Running".to_string(),
                                        has_active_run: true,
                                        func_run_id: *func_run_id,
                                        action_id: Some(*action_id),
                                    })
                                }
                                ActionState::Queued => {
                                    // Scheduled to run - show as queued
                                    Some(FunctionExecutionStatusV1 {
                                        state: "Queued".to_string(),
                                        has_active_run: true,
                                        func_run_id: *func_run_id,
                                        action_id: Some(*action_id),
                                    })
                                }
                                ActionState::OnHold => {
                                    // Waiting for dependencies - show as on hold
                                    Some(FunctionExecutionStatusV1 {
                                        state: "OnHold".to_string(),
                                        has_active_run: true,
                                        func_run_id: *func_run_id,
                                        action_id: Some(*action_id),
                                    })
                                }
                                ActionState::Failed => {
                                    // Function execution failed - always show as failed
                                    Some(FunctionExecutionStatusV1 {
                                        state: "Failed".to_string(),
                                        has_active_run: false,
                                        func_run_id: *func_run_id,
                                        action_id: Some(*action_id),
                                    })
                                }
                            };
                            
                            (execution_status, dependencies)
                        } else {
                            // No current action - check if there's action history to get the last result
                            let execution_status = if let Some(history) = action_history_map.get(&(*component_id, action_func.prototype_id)) {
                                if let Some(last_execution) = history.first() {
                                    // Use the last execution result as current status
                                    Some(FunctionExecutionStatusV1 {
                                        state: last_execution.state.clone(),
                                        has_active_run: false,
                                        func_run_id: Some(last_execution.func_run_id),
                                        action_id: None,
                                    })
                                } else {
                                    Some(FunctionExecutionStatusV1 {
                                        state: "Idle".to_string(),
                                        has_active_run: false,
                                        func_run_id: None,
                                        action_id: None,
                                    })
                                }
                            } else {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Idle".to_string(),
                                    has_active_run: false,
                                    func_run_id: None,
                                    action_id: None,
                                })
                            };
                            (execution_status, Vec::new())
                        };
                        
                        // Get execution history for this action function
                        let execution_history = if include_execution_history {
                            let history = action_history_map.get(&(*component_id, action_func.prototype_id))
                                .cloned()
                                .unwrap_or_else(Vec::new);
                            if !history.is_empty() {
                                info!("📊 [DEBUG] Found {} history entries for action function {} on component {}", 
                                    history.len(), action_func.func_name, component_id);
                            }
                            history
                        } else {
                            Vec::new()
                        };
                        
                        summary.action_functions.push(FunctionRelationshipV1 {
                            function_name: action_func.func_name,
                            execution_status,
                            depends_on,
                            execution_history,
                        });
                    }
                }
                
                // Add management functions with REAL execution state
                if include_management_functions {
                    for mgmt_func in management_functions {
                        // Check for pending/executing management function
                        let mgmt_execution = si_db::ManagementFuncJobState::get_latest_by_keys(
                            ctx,
                            *component_id,
                            mgmt_func.management_prototype_id,
                        ).await;
                        
                        let execution_status = match mgmt_execution {
                            Ok(Some(job_state)) => {
                                let func_run_id = job_state.func_run_id();
                                match job_state.state() {
                                    si_db::ManagementState::Pending => {
                                        Some(FunctionExecutionStatusV1 {
                                            state: "Pending".to_string(),
                                            has_active_run: true,
                                            func_run_id,
                                            action_id: None,
                                        })
                                    }
                                    si_db::ManagementState::Executing => {
                                        Some(FunctionExecutionStatusV1 {
                                            state: "Running".to_string(),
                                            has_active_run: true,
                                            func_run_id,
                                            action_id: None,
                                        })
                                    }
                                    si_db::ManagementState::Success => {
                                        Some(FunctionExecutionStatusV1 {
                                            state: "Succeeded".to_string(),
                                            has_active_run: false,
                                            func_run_id,
                                            action_id: None,
                                        })
                                    }
                                    si_db::ManagementState::Failure => {
                                        Some(FunctionExecutionStatusV1 {
                                            state: "Failed".to_string(),
                                            has_active_run: false,
                                            func_run_id,
                                            action_id: None,
                                        })
                                    }
                                    si_db::ManagementState::Operating => {
                                        Some(FunctionExecutionStatusV1 {
                                            state: "Operating".to_string(),
                                            has_active_run: true,
                                            func_run_id,
                                            action_id: None,
                                        })
                                    }
                                }
                            }
                            _ => {
                                Some(FunctionExecutionStatusV1 {
                                    state: "Available".to_string(),
                                    has_active_run: false,
                                    func_run_id: None,
                                    action_id: None,
                                })
                            }
                        };
                        
                        // Get execution history for this specific management function
                        let execution_history = if include_execution_history {
                            // Get ALL management job states for this component + prototype combination
                            let mut mgmt_history = Vec::new();
                            
                            // Query all job states for this component and prototype
                            let workspace_pk = ctx.tenancy().workspace_pk().unwrap_or_default();
                            let change_set_id = ctx.change_set_id();
                            
                            // Get management job states from database directly for this specific function
                            info!("📊 [DEBUG] Querying management history for component {} prototype {}", component_id, mgmt_func.management_prototype_id);
                            if let Ok(all_job_states) = ctx.txns().await?.pg().query(
                                "SELECT *, component_id as returned_component_id FROM management_func_job_states WHERE workspace_id = $1 AND component_id = $2 AND prototype_id = $3 ORDER BY created_at DESC LIMIT 10",
                                &[&workspace_pk, component_id, &mgmt_func.management_prototype_id]
                            ).await {
                                info!("📊 [DEBUG] Query returned {} job states", all_job_states.len());
                                for row in all_job_states {
                                    if let (Ok(func_run_id), Ok(state_str), Ok(created_at), Ok(returned_comp_id)) = (
                                        row.try_get::<_, Option<si_id::FuncRunId>>("func_run_id"),
                                        row.try_get::<_, String>("state"),
                                        row.try_get::<_, chrono::DateTime<chrono::Utc>>("created_at"),
                                        row.try_get::<_, ComponentId>("returned_component_id")
                                    ) {
                                        if returned_comp_id != *component_id {
                                            info!("📊 [DEBUG] WARNING: Query returned wrong component {} instead of {}", returned_comp_id, component_id);
                                        }
                                        if let Some(func_run_id) = func_run_id {
                                            mgmt_history.push(ExecutionHistoryEntry {
                                                func_run_id,
                                                state: state_str,
                                                started_at: created_at,
                                            });
                                        }
                                    }
                                }
                            }
                            
                            if !mgmt_history.is_empty() {
                                info!("📊 [DEBUG] Found {} history entries for management function {} on component {}", 
                                    mgmt_history.len(), mgmt_func.func_name, component_id);
                            }
                            mgmt_history
                        } else {
                            Vec::new()
                        };
                        
                        summary.management_functions.push(FunctionRelationshipV1 {
                            function_name: mgmt_func.func_name,
                            execution_status,
                            depends_on: Vec::new(), // Management functions don't have action dependencies
                            execution_history,
                        });
                    }
                }
                
                // Add qualification functions with REAL status
                if include_qualification_functions {
                    let qualification_avs = Component::list_qualification_avs(ctx, *component_id).await?;
                    
                    for qualification_av in &qualification_avs {
                        if let Ok(Some(qualification)) = dal::qualification::QualificationView::new(ctx, qualification_av.id()).await {
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
                                qualification_history_map.get(&(*component_id, qualification_av.id()))
                                    .cloned()
                                    .unwrap_or_else(Vec::new)
                            } else {
                                Vec::new()
                            };
                            
                            summary.qualification_functions.push(FunctionRelationshipV1 {
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
        
        info!("⚙️ [PERF] Function relationships completed in {}ms", functions_start.elapsed().as_millis());
    }
    
    info!("🏁 [PERF] Graph summary completed in {}ms", start_time.elapsed().as_millis());
    Ok(component_summaries)
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/graph/summary",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("limit" = Option<String>, Query, description = "Maximum number of components to return (default: 50, max: 300)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination"),
        ("includeSubscriptions" = Option<bool>, Query, description = "Include subscription relationships (default: true)"),
        ("includeManages" = Option<bool>, Query, description = "Include management relationships"),
        ("includeActionFunctions" = Option<bool>, Query, description = "Include action function relationships"),
        ("includeManagementFunctions" = Option<bool>, Query, description = "Include management function relationships"),
        ("includeQualificationFunctions" = Option<bool>, Query, description = "Include qualification function relationships"),
        ("includeResourceInfo" = Option<bool>, Query, description = "Include resource information (resource ID and status)"),
        ("includeDiffStatus" = Option<bool>, Query, description = "Include component diff status vs HEAD (Added/Modified/None)"),
        ("includeExecutionHistory" = Option<bool>, Query, description = "Include last 10 execution history entries for each function"),
        ("includeFunctions" = Option<bool>, Query, description = "Include all function types (convenience parameter)"),
        ("includeAll" = Option<bool>, Query, description = "Include everything (convenience parameter)"),
    ),
    summary = "Get component graph summary for visualization",
    tag = "graph",
    responses(
        (status = 200, description = "Graph summary retrieved successfully", body = ComponentRelationshipsV1Response, example = json!({
                    "graphSummary": {
                        "01H9ZQD35JPMBGHH69BT0Q79AA": {
                            "componentName": "vpc-component",
                            "schemaName": "AWS::EC2::VPC",
                            "hasResource": true,
                            "resourceId": "i-1234567890abcdef0",
                            "resourceStatus": "Ok",
                            "hasDiff": true,
                            "diffStatus": "Modified",
                            "subscriptions": [
                                {
                                    "toComponentId": "01H9ZQD35JPMBGHH69BT0Q79BB",
                                    "toComponentName": "subnet-component",
                                    "fromPath": "/domain/example",
                                    "toPath": "/domain/consume-me",
                                    "currentValue": "subnet-123"
                                }
                            ],
                            "actionFunctions": [
                                {
                                    "functionName": "Create Asset",
                                    "executionStatus": {
                                        "state": "Queued",
                                        "hasActiveRun": true,
                                        "actionId": "01H9ZQD35JPMBGHH69BT0Q79FF"
                                    },
                                    "dependsOn": []
                                },
                                {
                                    "functionName": "Update Asset",
                                    "executionStatus": {
                                        "state": "OnHold",
                                        "hasActiveRun": true,
                                        "actionId": "01H9ZQD35JPMBGHH69BT0Q79GG"
                                    },
                                    "dependsOn": ["01H9ZQD35JPMBGHH69BT0Q79FF"],
                                    "executionHistory": [
                                        {
                                            "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79HH",
                                            "state": "Failed",
                                            "startedAt": "2025-12-27T17:30:15Z"
                                        },
                                        {
                                            "funcRunId": "01H9ZQD35JPMBGHH69BT0Q79II",
                                            "state": "Succeeded",
                                            "startedAt": "2025-12-27T16:15:22Z"
                                        }
                                    ]
                                }
                            ]
                        }
                    },
                    "nextCursor": null
                })),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn graph_summary(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    axum::extract::Query(params): axum::extract::Query<ComponentRelationshipsParams>,
    tracker: PosthogEventTracker,
) -> Result<Json<ComponentRelationshipsV1Response>, ComponentsError> {
    let limit = params.limit.unwrap_or(50).min(300) as usize;
    let cursor = params.cursor;
    
    // Resolve what to include based on parameters
    let include_all = params.include_all.unwrap_or(false);
    let include_functions = params.include_functions.unwrap_or(false);
    
    let include_subscriptions = include_all || params.include_subscriptions.unwrap_or(true); // Default to true
    let include_manages = include_all || params.include_manages.unwrap_or(false);
    let include_action_functions = include_all || include_functions || params.include_action_functions.unwrap_or(false);
    let include_management_functions = include_all || include_functions || params.include_management_functions.unwrap_or(false);
    let include_qualification_functions = include_all || include_functions || params.include_qualification_functions.unwrap_or(false);
    let include_resource_info = include_all || params.include_resource_info.unwrap_or(false);
    let include_diff_status = include_all || params.include_diff_status.unwrap_or(false);
    let include_execution_history = include_all || params.include_execution_history.unwrap_or(false);
    
    // Get component summaries
    let component_summaries = build_graph_summary(
        ctx,
        include_subscriptions,
        include_manages,
        include_action_functions,
        include_management_functions,
        include_qualification_functions,
        include_resource_info,
        include_diff_status,
        include_execution_history,
    ).await?;

    // Handle pagination
    let component_ids: Vec<String> = component_summaries.keys().cloned().collect();
    let start_index = if let Some(ref cursor_str) = cursor {
        component_ids.iter().position(|comp_id| comp_id == cursor_str).map(|idx| idx + 1).unwrap_or(0)
    } else {
        0
    };
    
    let end_index = (start_index + limit).min(component_ids.len());
    let paginated_component_ids = &component_ids[start_index..end_index];
    
    let mut paginated_summaries: std::collections::BTreeMap<String, ComponentSummaryV1> = std::collections::BTreeMap::new();
    for component_id in paginated_component_ids {
        if let Some(summary) = component_summaries.get(component_id) {
            paginated_summaries.insert(component_id.clone(), summary.clone());
        }
    }
    
    let next_cursor = if end_index < component_ids.len() && !paginated_component_ids.is_empty() {
        paginated_component_ids.last().cloned()
    } else {
        None
    };

    tracker.track(ctx, "api_graph_summary", json!({}));

    Ok(Json(ComponentRelationshipsV1Response {
        graph_summary: paginated_summaries,
        next_cursor,
    }))
}
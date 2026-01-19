use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    hash::{
        Hash,
        Hasher,
    },
    time::Instant,
};

use chrono::{
    DateTime,
    Utc,
};
use dal::{
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
    action::{
        Action,
        dependency_graph::ActionDependencyGraph,
        prototype::ActionPrototype,
    },
    component::properties::ComponentProperties,
    diagram::Diagram,
};
use execution_histories::ExecutionHistories;
use si_events::ActionState;
use si_id::{
    ActionId,
    FuncRunId,
};
use telemetry::prelude::*;

pub(crate) mod execution_histories;

#[derive(Debug, Clone)]
pub struct ComponentSummary {
    pub component_name: String,
    pub schema_name: String,
    pub has_resource: Option<bool>,
    pub resource_id: Option<String>,
    pub resource_status: Option<String>,
    pub has_diff: Option<bool>,
    pub diff_status: Option<String>,
    pub subscriptions: Vec<SubscriptionRelationship>,
    pub manages: Vec<ManagementRelationship>,
    pub action_functions: Vec<FunctionRelationship>,
    pub management_functions: Vec<FunctionRelationship>,
    pub qualification_functions: Vec<FunctionRelationship>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionRelationship {
    pub to_component_id: ComponentId,
    pub to_component_name: String,
    pub from_path: String,
    pub to_path: String,
    pub current_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ManagementRelationship {
    pub to_component_id: ComponentId,
    pub to_component_name: String,
}

#[derive(Debug, Clone)]
pub struct FunctionRelationship {
    pub function_name: String,
    pub execution_status: Option<FunctionExecutionStatus>,
    pub depends_on: Vec<ActionId>,
    pub execution_history: Vec<ExecutionHistoryEntry>,
}

#[derive(Debug, Clone)]
pub struct FunctionExecutionStatus {
    pub state: String,
    pub has_active_run: bool,
    pub func_run_id: Option<FuncRunId>,
    pub action_id: Option<ActionId>,
}

#[derive(Debug, Clone)]
pub struct ExecutionHistoryEntry {
    pub func_run_id: FuncRunId,
    pub state: String,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ComponentSummaryGenerator {
    include_action_functions: bool,
    include_diff_status: bool,
    include_execution_history: bool,
    include_management_functions: bool,
    include_manages: bool,
    include_qualification_functions: bool,
    include_resource_info: bool,
    include_subscriptions: bool,
}

impl ComponentSummaryGenerator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        include_action_functions: bool,
        include_diff_status: bool,
        include_execution_history: bool,
        include_management_functions: bool,
        include_manages: bool,
        include_qualification_functions: bool,
        include_resource_info: bool,
        include_subscriptions: bool,
    ) -> Self {
        Self {
            include_action_functions,
            include_diff_status,
            include_execution_history,
            include_management_functions,
            include_manages,
            include_qualification_functions,
            include_resource_info,
            include_subscriptions,
        }
    }

    #[instrument(
        name = "dal_summary_generator.component.generate"
        level = "info",
        skip_all,
        fields(
            %self.include_action_functions,
            %self.include_diff_status,
            %self.include_execution_history,
            %self.include_management_functions,
            %self.include_manages,
            %self.include_qualification_functions,
            %self.include_resource_info,
            %self.include_subscriptions,
            count_action_lookup = Empty,
            count_components = %component_ids.len(),
            count_current_change_set_actions = Empty,
            count_head_actions = Empty,
            duration_ms_diff_status = Empty,
            duration_ms_execution_history = Empty,
            duration_ms_functions = Empty,
            duration_ms_manages = Empty,
            duration_ms_resource_info = Empty,
            duration_ms_subscriptions = Empty,
            duration_ms_summary = Empty,
        )
    )]
    pub async fn generate(
        &self,
        ctx: &DalContext,
        head_ctx: &DalContext,
        component_ids: &[ComponentId],
    ) -> Result<BTreeMap<String, ComponentSummary>, super::Error> {
        let span = current_span_for_instrument_at!("info");
        let start_time = Instant::now();

        let mut component_summaries: BTreeMap<String, ComponentSummary> = BTreeMap::new();

        // Initialize component summaries with basic info
        for component_id in component_ids {
            let component = Component::get_by_id(ctx, *component_id).await?;
            let component_name = component.name(ctx).await?;
            let schema_name = component.schema(ctx).await?.name;
            let component_id_str = component_id.to_string();

            component_summaries.insert(
                component_id_str,
                ComponentSummary {
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
        if self.include_resource_info {
            let resource_start = Instant::now();
            for component_id in component_ids {
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
            span.record(
                "duration_ms_resource_info",
                resource_start.elapsed().as_millis(),
            );
        }

        // Add diff status information if requested
        if self.include_diff_status {
            let diff_start = Instant::now();

            for component_id in component_ids {
                let exists_in_head = Component::exists_by_id(head_ctx, *component_id).await?;
                let exists_in_changeset = Component::exists_by_id(ctx, *component_id).await?;

                let (has_diff, diff_status) = if !exists_in_head && exists_in_changeset {
                    (true, "Added")
                } else if exists_in_head && !exists_in_changeset {
                    (true, "Removed")
                } else if exists_in_head && exists_in_changeset {
                    let has_changes =
                        Self::has_meaningful_component_changes(ctx, head_ctx, *component_id)
                            .await?;
                    (has_changes, if has_changes { "Modified" } else { "None" })
                } else {
                    (false, "None")
                };

                if let Some(summary) = component_summaries.get_mut(&component_id.to_string()) {
                    summary.has_diff = Some(has_diff);
                    summary.diff_status = Some(diff_status.to_string());
                }
            }
            span.record("duration_ms_diff_status", diff_start.elapsed().as_millis());
        }

        // Add subscription relationships if requested
        if self.include_subscriptions {
            let subscriptions_start = Instant::now();

            let diagram = Diagram::assemble(ctx, None).await?;

            for edge in &diagram.attribute_subscription_edges {
                let to_name = Component::name_by_id(ctx, edge.to_component_id).await?;
                let current_value = AttributeValue::view(ctx, edge.to_attribute_value_id).await?;

                let from_component_str = edge.from_component_id.to_string();
                if let Some(summary) = component_summaries.get_mut(&from_component_str) {
                    summary.subscriptions.push(SubscriptionRelationship {
                        to_component_id: edge.to_component_id,
                        to_component_name: to_name,
                        from_path: edge.from_attribute_path.clone(),
                        to_path: edge.to_attribute_path.clone(),
                        current_value,
                    });
                }
            }
            span.record(
                "duration_ms_subscriptions",
                subscriptions_start.elapsed().as_millis(),
            );
        }

        // Add management relationships if requested
        if self.include_manages {
            let manages_start = Instant::now();

            for component_id in component_ids {
                let component = Component::get_by_id(ctx, *component_id).await?;
                let managed_components = component.get_managed(ctx).await?;

                for managed_id in managed_components {
                    let to_name = Component::name_by_id(ctx, managed_id).await?;
                    let from_component_str = component_id.to_string();

                    if let Some(summary) = component_summaries.get_mut(&from_component_str) {
                        summary.manages.push(ManagementRelationship {
                            to_component_id: managed_id,
                            to_component_name: to_name,
                        });
                    }
                }
            }
            span.record("duration_ms_manages", manages_start.elapsed().as_millis());
        }

        let mut execution_histories = ExecutionHistories::default();
        if self.include_execution_history {
            let execution_history_start = Instant::now();
            execution_histories
                .populate(
                    ctx,
                    head_ctx,
                    self.include_action_functions,
                    self.include_management_functions,
                    self.include_qualification_functions,
                )
                .await?;
            span.record(
                "duration_ms_execution_history",
                execution_history_start.elapsed().as_millis(),
            );
        }
        let action_history_map = execution_histories.action_history_map;
        let management_history_map = execution_histories.management_history_map;
        let qualification_history_map = execution_histories.qualification_history_map;

        // Add function relationships based on specific parameters
        let need_any_functions = self.include_action_functions
            || self.include_management_functions
            || self.include_qualification_functions;
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
            let mut current_change_set_actions = Vec::new();
            for component_id in component_ids {
                if let Ok(actions) = Action::find_for_component_id(ctx, *component_id).await {
                    current_change_set_actions.extend(actions);
                }
            }
            span.record(
                "count_current_change_set_actions",
                current_change_set_actions.len(),
            );

            for action_id in current_change_set_actions {
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
            let current_change_set_id = ctx.change_set_id();
            let head_change_set_id = head_ctx.change_set_id();

            if current_change_set_id != head_change_set_id {
                let head_actions = Action::list_topologically(head_ctx).await?;
                span.record("count_head_actions", head_actions.len());

                for action_id in head_actions {
                    if let (Ok(Some(comp_id)), Ok(proto_id), Ok(action)) = (
                        Action::component_id(head_ctx, action_id).await,
                        Action::prototype_id(head_ctx, action_id).await,
                        Action::get_by_id(head_ctx, action_id).await,
                    ) {
                        // Only add HEAD actions if we don't already have a current action for this component+prototype
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            action_lookup.entry((comp_id, proto_id))
                        {
                            let func_run_id =
                                Action::last_func_run_id_for_id_opt(head_ctx, action_id).await?;
                            e.insert((action.state(), func_run_id, action_id));
                        }
                    }
                }
            }

            span.record("count_action_lookup", action_lookup.len());

            // Process each component to add function data
            for component_id in component_ids {
                if let Some(summary) = component_summaries.get_mut(&component_id.to_string()) {
                    let component = Component::get_by_id(ctx, *component_id).await?;

                    // Add action functions with REAL status
                    if self.include_action_functions {
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
                                if let Some((state, func_run_id, action_id)) =
                                    action_lookup.get(&key)
                                {
                                    // There's a real action scheduled
                                    let execution_history = if self.include_execution_history {
                                        action_history_map
                                            .get(&key)
                                            .cloned()
                                            .unwrap_or_else(Vec::new)
                                    } else {
                                        Vec::new()
                                    };

                                    let depends_on: Vec<ActionId> = action_dependency_graph
                                        .direct_dependencies_of(*action_id)
                                        .into_iter()
                                        .collect();

                                    let execution_status = Some(FunctionExecutionStatus {
                                        state: format!("{state:?}"),
                                        has_active_run: *state == ActionState::Running
                                            || *state == ActionState::Dispatched,
                                        func_run_id: *func_run_id,
                                        action_id: Some(*action_id),
                                    });

                                    summary.action_functions.push(FunctionRelationship {
                                        function_name: prototype.name.clone(),
                                        execution_status,
                                        depends_on,
                                        execution_history,
                                    });
                                } else {
                                    // No action scheduled - show as Idle
                                    let execution_history = if self.include_execution_history {
                                        action_history_map
                                            .get(&key)
                                            .cloned()
                                            .unwrap_or_else(Vec::new)
                                    } else {
                                        Vec::new()
                                    };

                                    let execution_status = Some(FunctionExecutionStatus {
                                        state: "Idle".to_string(),
                                        has_active_run: false,
                                        func_run_id: None,
                                        action_id: None,
                                    });

                                    summary.action_functions.push(FunctionRelationship {
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
                    if self.include_management_functions {
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
                                let execution_history = if self.include_execution_history {
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
                                    Some(FunctionExecutionStatus {
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
                                    Some(FunctionExecutionStatus {
                                        state: "Available".to_string(),
                                        has_active_run: false,
                                        func_run_id: None,
                                        action_id: None,
                                    })
                                };

                                summary.management_functions.push(FunctionRelationship {
                                    function_name: mgmt_prototype.name.clone(),
                                    execution_status,
                                    depends_on: Vec::new(), // Management functions don't have action dependencies
                                    execution_history,
                                });
                            }
                        }
                    }

                    // Add qualification functions with REAL status
                    if self.include_qualification_functions {
                        let qualification_avs =
                            Component::list_qualification_avs(ctx, *component_id).await?;

                        for qualification_av in &qualification_avs {
                            if let Ok(Some(qualification)) =
                                dal::qualification::QualificationView::new(
                                    ctx,
                                    qualification_av.id(),
                                )
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
                                            Some(FunctionExecutionStatus{
                                                state: "Succeeded".to_string(),
                                                has_active_run: false,
                                                func_run_id,
                                                action_id: None,
                                            })
                                        }
                                        dal::qualification::QualificationSubCheckStatus::Failure => {
                                            Some(FunctionExecutionStatus {
                                                state: "Failed".to_string(),
                                                has_active_run: false,
                                                func_run_id,
                                                action_id: None,
                                            })
                                        }
                                        dal::qualification::QualificationSubCheckStatus::Warning => {
                                            Some(FunctionExecutionStatus{
                                                state: "Warning".to_string(),
                                                has_active_run: false,
                                                func_run_id,
                                                action_id: None,
                                            })
                                        }
                                        _ => {
                                            Some(FunctionExecutionStatus{
                                                state: "Completed".to_string(),
                                                has_active_run: false,
                                                func_run_id,
                                                action_id: None,
                                            })
                                        }
                                    }
                                    } else {
                                        Some(FunctionExecutionStatus {
                                            state: "Completed".to_string(),
                                            has_active_run: false,
                                            func_run_id,
                                            action_id: None,
                                        })
                                    }
                                } else {
                                    Some(FunctionExecutionStatus {
                                        state: "Running".to_string(),
                                        has_active_run: true,
                                        func_run_id,
                                        action_id: None,
                                    })
                                };

                                // Get execution history for this qualification function
                                let execution_history = if self.include_execution_history {
                                    qualification_history_map
                                        .get(&(*component_id, qualification_av.id()))
                                        .cloned()
                                        .unwrap_or_else(Vec::new)
                                } else {
                                    Vec::new()
                                };

                                summary.qualification_functions.push(FunctionRelationship {
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

            span.record(
                "duration_ms_functions",
                functions_start.elapsed().as_millis(),
            );
        }

        // Record the overall duration!
        span.record("duration_ms_summary", start_time.elapsed().as_millis());

        Ok(component_summaries)
    }

    // Hash-based component diff check
    async fn has_meaningful_component_changes(
        ctx: &DalContext,
        head_ctx: &DalContext,
        component_id: ComponentId,
    ) -> Result<bool, super::Error> {
        use std::collections::hash_map::DefaultHasher;

        let exists_in_changeset = Component::exists_by_id(ctx, component_id).await?;
        let exists_in_head = Component::exists_by_id(head_ctx, component_id).await?;

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
            if let Some(view) = Component::view_by_id(head_ctx, component_id).await? {
                if let Ok(props) = ComponentProperties::try_from(view) {
                    if let Ok(json_str) = serde_json::to_string(&props) {
                        json_str.hash(&mut hasher);
                    }
                }
            }
            if let Ok(action_ids) = Action::find_for_component_id(head_ctx, component_id).await {
                for action_id in action_ids {
                    if let Ok(action) = Action::get_by_id(head_ctx, action_id).await {
                        action.state().hash(&mut hasher);
                        if let Ok(proto_id) = Action::prototype_id(head_ctx, action_id).await {
                            proto_id.hash(&mut hasher);
                        }
                    }
                }
            }
            hasher.finish()
        };

        Ok(changeset_hash != head_hash)
    }
}

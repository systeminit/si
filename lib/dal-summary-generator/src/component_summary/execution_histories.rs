use std::collections::HashMap;

use dal::{
    AttributeValueId,
    ComponentId,
    DalContext,
    action::{
        Action,
        prototype::ActionPrototype,
    },
};
use si_events::{
    ActionKind,
    ActionResultState,
    FuncRun,
    FuncRunState,
};
use si_id::ManagementPrototypeId;
use telemetry::prelude::*;

use super::ExecutionHistoryEntry;

#[derive(Default, Debug)]
pub(crate) struct ExecutionHistories {
    pub(crate) action_history_map:
        HashMap<(ComponentId, dal::ActionPrototypeId), Vec<ExecutionHistoryEntry>>,
    pub(crate) management_history_map:
        HashMap<(ComponentId, ManagementPrototypeId), Vec<ExecutionHistoryEntry>>,
    pub(crate) qualification_history_map:
        HashMap<(ComponentId, AttributeValueId), Vec<ExecutionHistoryEntry>>,
}

impl ExecutionHistories {
    #[instrument(
        name = "dal_summary_generator.component.execution_histories.populate"
        level = "info",
        skip_all,
        fields(
            %include_action_functions,
            %include_management_functions,
            %include_qualification_functions,
            head_action_runs_count = Empty,
            refresh_action_runs_count = Empty,
            action_history_map_len = Empty
        )
    )]
    pub(crate) async fn populate(
        &mut self,
        ctx: &DalContext,
        head_ctx: &DalContext,
        include_action_functions: bool,
        include_management_functions: bool,
        include_qualification_functions: bool,
    ) -> Result<(), crate::Error> {
        let span = current_span_for_instrument_at!("info");

        let workspace_pk = ctx.tenancy().workspace_pk().unwrap_or_default();

        // Build action execution history using DUAL CONTEXT approach
        if include_action_functions {
            // Create HEAD context for regular action history (Create/Update/Delete)
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

                span.record("head_action_runs_count", head_action_runs.len());

                for func_run in head_action_runs {
                    if let Some(action_id) = func_run.action_id() {
                        if let (Ok(Some(comp_id)), Ok(proto_id)) = (
                            Action::component_id(head_ctx, action_id).await,
                            Action::prototype_id(head_ctx, action_id).await,
                        ) {
                            let result_state = Self::get_action_result_from_func_run(&func_run);

                            let entry = ExecutionHistoryEntry {
                                func_run_id: func_run.id(),
                                state: result_state,
                                started_at: func_run.created_at(),
                            };

                            self.action_history_map
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

                span.record("refresh_action_runs_count", refresh_action_runs.len());

                for func_run in refresh_action_runs {
                    if let Some(action_id) = func_run.action_id() {
                        // Check if this is a refresh action by trying current context first
                        if let (Ok(Some(comp_id)), Ok(proto_id)) = (
                            Action::component_id(ctx, action_id).await,
                            Action::prototype_id(ctx, action_id).await,
                        ) {
                            // Get the action prototype to check if it's a refresh action
                            if let Ok(prototype) = ActionPrototype::get_by_id(ctx, proto_id).await {
                                if ActionKind::Refresh == prototype.kind.into() {
                                    let result_state =
                                        Self::get_action_result_from_func_run(&func_run);

                                    let entry = ExecutionHistoryEntry {
                                        func_run_id: func_run.id(),
                                        state: result_state,
                                        started_at: func_run.created_at(),
                                    };

                                    self.action_history_map
                                        .entry((comp_id, proto_id))
                                        .or_default()
                                        .push(entry);
                                }
                            }
                        }
                    }
                }
            }

            span.record("action_history_map_len", self.action_history_map.len());
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

                        self.qualification_history_map
                            .entry((comp_id, av_id))
                            .or_default()
                            .push(entry);
                    }
                }
            }

            // Sort and limit qualification history
            for history in self.qualification_history_map.values_mut() {
                history.sort_by(|a, b| b.started_at.cmp(&a.started_at));
                history.truncate(10);
            }
        }

        Ok(())
    }

    // Helper function for action execution status
    fn get_action_result_from_func_run(func_run: &FuncRun) -> String {
        if func_run.state() == FuncRunState::Failure {
            return "Failed".to_string();
        }

        match func_run.action_result_state() {
            Some(ActionResultState::Success) => "Succeeded".to_string(),
            Some(ActionResultState::Failure) => "Failed".to_string(),
            Some(ActionResultState::Unknown) => "Unknown".to_string(),
            None => match func_run.state() {
                FuncRunState::Success => "Succeeded".to_string(),
                FuncRunState::Failure => "Failed".to_string(),
                _ => "Idle".to_string(),
            },
        }
    }
}

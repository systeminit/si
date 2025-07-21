use axum::{
    Json,
    extract::Path,
};
use dal::{
    ChangeSetId,
    WorkspacePk,
};
use sdf_extract::change_set::ChangeSetDalContext;
use si_db::{
    ManagementFuncJobState,
    ManagementState,
};
use si_events::FuncRunId;

use super::ManagementApiResult;

pub async fn state(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    Path((_workspace_pk, _change_set_id, func_run_id)): Path<(WorkspacePk, ChangeSetId, FuncRunId)>,
) -> ManagementApiResult<Json<Option<si_frontend_types::ManagementFuncJobState>>> {
    let state = ManagementFuncJobState::get_latest_by_func_run_id(ctx, func_run_id)
        .await?
        .map(|row| si_frontend_types::ManagementFuncJobState {
            id: row.id(),
            workspace_id: row.workspace_id(),
            change_set_id: row.change_set_id(),
            component_id: row.component_id(),
            prototype_id: row.prototype_id(),
            user_id: row.user_id(),
            func_run_id: row.func_run_id(),
            state: match row.state() {
                ManagementState::Executing => si_frontend_types::ManagementState::Executing,
                ManagementState::Failure => si_frontend_types::ManagementState::Failure,
                ManagementState::Operating => si_frontend_types::ManagementState::Operating,
                ManagementState::Pending => si_frontend_types::ManagementState::Pending,
                ManagementState::Success => si_frontend_types::ManagementState::Success,
            },
            timestamp: row.timestamp(),
            message: row.message().clone(),
        });
    Ok(Json(state))
}

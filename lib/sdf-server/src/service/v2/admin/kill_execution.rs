use axum::{extract::Path, response::IntoResponse};
use dal::func::runner::FuncRunner;
use si_events::FuncRunId;

use crate::service::v2::admin::{AdminAPIResult, AdminUserContext};

pub async fn kill_execution(
    AdminUserContext(ctx): AdminUserContext,
    Path(func_run_id): Path<FuncRunId>,
) -> AdminAPIResult<impl IntoResponse> {
    FuncRunner::kill_execution(&ctx, func_run_id).await?;

    // We commit without a rebase here because we need to commit our func run table changes.
    ctx.commit_no_rebase().await?;

    Ok(())
}

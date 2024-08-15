use axum::{extract::Path, response::IntoResponse};
use dal::func::runner::FuncRunner;
use si_events::{FuncRunId, WorkspacePk};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::AdminAPIResult;

pub async fn cancel_execution(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, func_run_id)): Path<(WorkspacePk, FuncRunId)>,
) -> AdminAPIResult<impl IntoResponse> {
    let ctx = builder.build_head(access_builder).await?;

    FuncRunner::cancel_execution(&ctx, func_run_id).await?;

    // We commit without a rebase here because we need to commit our func run table changes.
    ctx.commit_no_rebase().await?;

    Ok(())
}

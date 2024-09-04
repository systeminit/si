use axum::{extract::Path, response::IntoResponse};
use dal::func::runner::FuncRunner;
use si_events::FuncRunId;

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::AdminAPIResult;

pub async fn kill_execution(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path(func_run_id): Path<FuncRunId>,
) -> AdminAPIResult<impl IntoResponse> {
    let ctx = builder.build_head(access_builder).await?;

    FuncRunner::kill_execution(&ctx, func_run_id).await?;

    // We commit without a rebase here because we need to commit our func run table changes.
    ctx.commit_no_rebase().await?;

    Ok(())
}

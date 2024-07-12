use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use dal::{ChangeSetId, Func, WorkspacePk};

use super::FuncAPIResult;

pub async fn list_all_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> FuncAPIResult<Json<Vec<Func>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let funcs = Func::list_all(&ctx).await?;
    Ok(Json(funcs))
}

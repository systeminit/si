use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use dal::{ChangeSetId, Func, WorkspacePk};
use telemetry::prelude::*;

use super::FuncAPIResult;
use crate::extract::{AccessBuilder, HandlerContext, PosthogClient};

pub async fn list_all_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> FuncAPIResult<Json<Vec<si_frontend_types::FuncSummary>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let mut funcs = Vec::new();

    for func in Func::list_all(&ctx).await? {
        match func.into_frontend_type(&ctx).await {
            Ok(f) => {
                funcs.push(f);
            }
            Err(err) => {
                error!(
                    ?err,
                    "could not make func with id {} into frontend type", func.id
                )
            }
        }
    }
    Ok(Json(funcs))
}

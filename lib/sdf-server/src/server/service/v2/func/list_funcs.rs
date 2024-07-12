use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use dal::{ChangeSetId, Func, WorkspacePk};
use si_frontend_types as frontend_types;
use telemetry::prelude::*;

use super::FuncAPIResult;

pub async fn list_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> FuncAPIResult<Json<Vec<frontend_types::FuncSummary>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let mut funcs = Vec::new();

    for func in Func::list_for_default_and_editing(&ctx).await? {
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

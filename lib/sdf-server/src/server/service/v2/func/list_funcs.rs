use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use dal::{ChangeSetId, Func, FuncId, WorkspacePk};

use serde::{Deserialize, Serialize};
use si_frontend_types as frontend_types;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};

use super::FuncAPIResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRequest {
    pub func_ids: Vec<FuncId>,
}

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

    for func in Func::list(&ctx).await? {
        funcs.push(func.into_frontend_type(&ctx).await?);
    }
    Ok(Json(funcs))
}

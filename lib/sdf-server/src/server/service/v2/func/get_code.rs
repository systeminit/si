use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use dal::{ChangeSetId, FuncId, WorkspacePk};

use serde::{Deserialize, Serialize};
use si_frontend_types::FuncCode;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};

use super::{get_code_response, FuncAPIResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRequest {
    pub func_ids: Vec<FuncId>,
}

pub async fn get_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<GetRequest>,
) -> FuncAPIResult<Json<Vec<FuncCode>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let mut funcs = Vec::new();

    for func_id in request.func_ids {
        funcs.push(get_code_response(&ctx, func_id).await?);
    }
    Ok(Json(funcs))
}

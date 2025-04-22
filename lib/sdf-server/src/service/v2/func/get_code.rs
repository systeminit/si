use axum::{
    Json,
    extract::{
        OriginalUri,
        Path,
        Query,
    },
};
use dal::{
    ChangeSetId,
    FuncId,
    WorkspacePk,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_types::FuncCode;

use super::{
    FuncAPIResult,
    get_code_response,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::v2::AccessBuilder,
};

// TODO: find the right way to pass a Vec<FuncId>
// the API call uses the `id[]=<...>&id[]=<...?` format
// but that doesn't work here with Rust
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRequest {
    pub id: FuncId,
}

pub async fn get_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Query(request): Query<GetRequest>,
) -> FuncAPIResult<Json<Vec<FuncCode>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let mut funcs = Vec::new();

    funcs.push(get_code_response(&ctx, request.id).await?);
    Ok(Json(funcs))
}

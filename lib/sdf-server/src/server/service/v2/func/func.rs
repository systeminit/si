use axum::{
    extract::{OriginalUri, Path},
    response::IntoResponse,
    routing::get,
    Json,
};
use dal::{
    func::authoring::FuncAuthoringClient, ChangeSet, ChangeSetId, Func, FuncId, WorkspacePk,
};
use si_frontend_types::{self as frontend_types, FuncBindings, FuncCode};

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    state::AppState,
    tracking::track,
};

use super::{ApiError, FuncAPIResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFuncRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncRequest {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub bindings: Vec<FuncBindings>,
    pub kind: FuncK,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRequest {
    pub func_ids: Vec<FuncId>,
}

pub async fn list_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
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

pub async fn get_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<GetRequest>,
) -> FuncAPIResult<Json<Vec<frontend_types::FuncCode>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let mut func_codes = vec![];
    for func_id in request.func_ids {
        let func = Func::get_by_id_or_error(&ctx, func_id).await?;
        func_codes.push(FuncCode {
            func_id: func.id.into(),
            code: func.code_plaintext().into(),
            types: [
                FuncAuthoringClient::compile_return_types(
                    func.backend_response_type,
                    func.backend_kind,
                ),
                &input_type,
                FuncAuthoringClient::compile_langjs_types(),
            ]
            .join("\n"),
        });
    }

    Ok(Json(func_codes))
}
pub async fn create(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<UpdateFuncRequest>,
) -> FuncAPIResult<impl IntoResponse> {
}

pub async fn update(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<UpdateFuncRequest>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::update_func(&ctx, func_id, request.display_name, request.description)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}

pub async fn save_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<String>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::save_code(&ctx, func_id, request).await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}

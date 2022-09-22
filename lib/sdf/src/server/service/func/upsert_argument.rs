use super::{FuncResult, FuncError};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use serde::{Deserialize, Serialize};
use dal::{StandardModel, func::argument::{FuncArgument, FuncArgumentId, FuncArgumentKind}}; 

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpsertArgumentRequest {
    pub func_argument_id: Option<FuncArgumentId>,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    pub name: String,
    pub func_id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub struct UpsertArgumentResponse {
    pub success: boolean;
}

pub async fn upsert_func_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<UpsertArgumentRequest>,
) -> FuncResult<Json<UpsertArgumentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // Check existence of func id
    let _func = Func::get_by_id(&ctx, request.func_id).await?.ok_or_else(FuncError::FuncNotFound)?;

    let existing_with_name = {
        match FuncArgument::find_by_attr(&ctx, "name", &request.name).await? {
            None => None,
            Some(existing) => if existing.visibility().is_head() { None } else { Some(existing) }
        }
    };

    match existing {
        Some(existing) => {
            existing.
}


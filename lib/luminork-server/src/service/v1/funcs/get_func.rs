use axum::{
    Json,
    extract::Path,
};
use dal::{
    Func,
    func::FuncKind,
};
use serde::Serialize;
use utoipa::ToSchema;

use super::{
    FuncV1RequestPath,
    FuncsError,
    FuncsResult,
};
use crate::extract::change_set::ChangeSetDalContext;

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id}",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
        ("func_id", description = "Func identifier"),
    ),
    tag = "funcs",
    responses(
        (status = 200, description = "Func retrieved successfully", body = GetFuncV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Func not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_func(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Path(FuncV1RequestPath { func_id }): Path<FuncV1RequestPath>,
) -> FuncsResult<Json<GetFuncV1Response>> {
    let func = Func::get_by_id_opt(ctx, func_id)
        .await?
        .ok_or(FuncsError::FuncNotFound(func_id))?;

    Ok(Json(GetFuncV1Response {
        name: func.clone().name,
        description: func.clone().description,
        display_name: func.clone().display_name,
        link: func.clone().link,
        is_locked: func.is_locked,
        kind: func.kind,
        code: func.code_plaintext()?.unwrap_or("".to_string()),
    }))
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncV1Response {
    #[schema(value_type = String)]
    pub name: String,
    #[schema(value_type = String)]
    pub description: Option<String>,
    #[schema(value_type = String)]
    pub display_name: Option<String>,
    #[schema(value_type = String)]
    pub kind: FuncKind,
    #[schema(value_type = bool)]
    pub is_locked: bool,
    #[schema(value_type = String)]
    pub code: String,
    #[schema(value_type = String)]
    pub link: Option<String>,
}

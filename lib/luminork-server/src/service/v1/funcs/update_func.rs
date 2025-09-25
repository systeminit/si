use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Func,
    func::authoring::FuncAuthoringClient,
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    FuncV1RequestPath,
    FuncsError,
    FuncsResult,
};

#[utoipa::path(
    put,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("func_id" = String, Path, description = "Func identifier"),
    ),
    summary = "Update a func",
    tag = "funcs",
    request_body = UpdateFuncV1Request,
    responses(
        (status = 200, description = "Function successfully updated", body = UpdateFuncV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Function not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn update_func(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(FuncV1RequestPath { func_id }): Path<FuncV1RequestPath>,
    payload: Result<Json<UpdateFuncV1Request>, axum::extract::rejection::JsonRejection>,
) -> FuncsResult<Json<UpdateFuncV1Response>> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(FuncsError::NotPermittedOnHead);
    }

    let func = Func::get_by_id(ctx, func_id).await?;

    if func.is_locked {
        return Err(FuncsError::LockedFunc(func_id));
    }

    if payload.display_name.clone() != func.display_name || payload.description != func.description
    {
        let updated_func = FuncAuthoringClient::update_func(
            ctx,
            func_id,
            payload.display_name.clone(),
            payload.description,
        )
        .await?;

        ctx.write_audit_log(
            AuditLogKind::UpdateFuncMetadata {
                func_id,
                old_display_name: func.display_name,
                new_display_name: updated_func.display_name.clone(),
                old_description: func.description,
                new_description: updated_func.description.clone(),
            },
            updated_func.name.clone(),
        )
        .await?;

        tracker.track(
            ctx,
            "api_update_func",
            serde_json::json!({
                "func_id": func_id,
                "func_name": updated_func.name.clone(),
                "func_kind": updated_func.kind.clone(),
            }),
        );
    }

    FuncAuthoringClient::save_code(ctx, func_id, payload.code).await?;
    tracker.track(
        ctx,
        "api_save_func_code",
        serde_json::json!({
            "func_id": func_id,
            "func_name": payload.display_name.clone(),
            "func_kind": func.kind.clone(),
        }),
    );

    ctx.commit().await?;

    Ok(Json(UpdateFuncV1Response { success: true }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFuncV1Request {
    #[schema(value_type = Option<String>, example = "Updated Display Name")]
    pub display_name: Option<String>,
    #[schema(value_type = Option<String>, example = "Updated Description")]
    pub description: Option<String>,
    #[schema(value_type = String, example = "<!-- String escaped Typescript code here -->")]
    pub code: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFuncV1Response {
    #[schema(value_type = bool)]
    pub success: bool,
}

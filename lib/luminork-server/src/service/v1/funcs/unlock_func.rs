use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Func,
    FuncId,
    SchemaVariant,
    SchemaVariantId,
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
use serde_json::json;
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
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id}/unlock",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("func_id" = String, Path, description = "Func identifier"),
    ),
    tag = "funcs",
    request_body = UnlockFuncV1Request,
    summary = "Unlocks a func - if there's already an unlocked function, then we return that",
    responses(
        (status = 200, description = "Function unlocked successfully", body = UnlockFuncV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn unlock_func(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(FuncV1RequestPath { func_id }): Path<FuncV1RequestPath>,
    payload: Result<Json<UnlockFuncV1Request>, axum::extract::rejection::JsonRejection>,
) -> FuncsResult<Json<UnlockFuncV1Response>> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(FuncsError::NotPermittedOnHead);
    }

    let schema_variant = SchemaVariant::get_by_id(ctx, payload.schema_variant_id).await?;
    if schema_variant.is_locked() {
        return Err(FuncsError::SchemaVariantMustBeUnlocked);
    }

    let func = Func::get_by_id_opt(ctx, func_id)
        .await?
        .ok_or(FuncsError::FuncNotFound(func_id))?;

    if !func.is_locked {
        return Ok(Json(UnlockFuncV1Response {
            unlocked_func_id: func.id,
        }));
    }

    let unlocked_func = FuncAuthoringClient::create_unlocked_func_copy(
        ctx,
        func_id,
        Some(payload.schema_variant_id),
    )
    .await?;

    ctx.write_audit_log(
        AuditLogKind::UnlockFunc {
            func_id,
            func_display_name: unlocked_func.display_name.clone(),
            schema_variant_id: Some(schema_variant.id),
            component_id: None,
            subject_name: Some(schema_variant.display_name().to_string()),
        },
        unlocked_func.name.clone(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_unlock_func",
        json!({
            "schema_variant_id": schema_variant.id,
            "unlocked_func_id": unlocked_func.id,
        }),
    );

    ctx.commit().await?;

    Ok(Json(UnlockFuncV1Response {
        unlocked_func_id: unlocked_func.id,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnlockFuncV1Request {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q75XY")]
    pub schema_variant_id: SchemaVariantId,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnlockFuncV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q75XY")]
    pub unlocked_func_id: FuncId,
}

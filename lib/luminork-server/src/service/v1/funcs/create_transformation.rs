use axum::response::Json;
use dal::{
    FuncId,
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
    FuncsError,
    FuncsResult,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/transformation",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    summary = "Create a transformation function",
    tag = "funcs",
    request_body = CreateTransformationFuncV1Request,
    responses(
        (status = 200, description = "Transformation function successfully created", body = CreateTransformationFuncV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_transformation(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<
        Json<CreateTransformationFuncV1Request>,
        axum::extract::rejection::JsonRejection,
    >,
) -> FuncsResult<Json<CreateTransformationFuncV1Response>> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(FuncsError::NotPermittedOnHead);
    }

    let func = FuncAuthoringClient::create_new_transformation_func(ctx, Some(payload.name.clone()))
        .await?;

    FuncAuthoringClient::update_func(
        ctx,
        func.id,
        payload.display_name.clone(),
        payload.description.clone(),
    )
    .await?;

    FuncAuthoringClient::save_code(ctx, func.id, payload.code).await?;

    ctx.write_audit_log(
        AuditLogKind::CreateFunc {
            func_display_name: func.display_name.clone(),
            func_kind: func.kind.into(),
        },
        func.name.clone(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_create_transformation_func",
        serde_json::json!({
            "func_id": func.id,
            "func_name": func.name.to_owned(),
        }),
    );

    ctx.commit().await?;

    Ok(Json(CreateTransformationFuncV1Response {
        func_id: func.id,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransformationFuncV1Request {
    #[schema(value_type = String, example = "myTransformation")]
    pub name: String,
    #[schema(value_type = Option<String>, example = "My Transformation")]
    pub display_name: Option<String>,
    #[schema(value_type = Option<String>, example = "A custom transformation function")]
    pub description: Option<String>,
    #[schema(value_type = String, example = "<!-- String escaped Typescript code here -->")]
    pub code: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransformationFuncV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub func_id: FuncId,
}

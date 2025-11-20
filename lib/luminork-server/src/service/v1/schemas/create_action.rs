use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    FuncId,
    SchemaVariant,
    action::prototype::ActionKind,
    cached_module::CachedModule,
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
    SchemaError,
    SchemaResult,
    SchemaVariantV1RequestPath,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/action",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
        ("schema_variant_id" = String, Path, description = "Schema variant identifier"),
    ),
    summary = "Create an action function and attach to a schema variant",
    tag = "schemas",
    request_body = CreateVariantActionFuncV1Request,
    responses(
        (status = 200, description = "Action function successfully created and attached to the variant", body = CreateVariantActionFuncV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 412, description = "Schema variant not found for schema"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_variant_action(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SchemaVariantV1RequestPath {
        schema_id,
        schema_variant_id,
    }): Path<SchemaVariantV1RequestPath>,
    payload: Result<
        Json<CreateVariantActionFuncV1Request>,
        axum::extract::rejection::JsonRejection,
    >,
) -> SchemaResult<Json<CreateVariantActionFuncV1Response>> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(SchemaError::NotPermittedOnHead);
    }

    let schema_id_for_variant: dal::SchemaId =
        SchemaVariant::schema_id(ctx, schema_variant_id).await?;
    if schema_id != schema_id_for_variant {
        return Err(SchemaError::SchemaVariantNotMemberOfSchema(
            schema_id,
            schema_variant_id,
        ));
    }

    let skip_overlay = payload.skip_overlay.unwrap_or(false);

    let is_builtin = CachedModule::find_latest_for_schema_id(ctx, schema_id)
        .await?
        .is_some();

    let func = if is_builtin && !skip_overlay {
        FuncAuthoringClient::create_new_action_func_overlay(
            ctx,
            Some(payload.name),
            payload.kind,
            schema_id,
        )
        .await?
    } else {
        let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
        if schema_variant.is_locked() {
            return Err(SchemaError::LockedVariant(schema_variant_id));
        }

        FuncAuthoringClient::create_new_action_func(
            ctx,
            Some(payload.name),
            payload.kind,
            schema_variant_id,
        )
        .await?
    };

    FuncAuthoringClient::update_func(ctx, func.id, payload.display_name, payload.description)
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
    ctx.write_audit_log(
        AuditLogKind::AttachActionFunc {
            func_id: func.id,
            func_display_name: func.display_name.clone(),
            schema_variant_id: Some(schema_variant_id),
            component_id: None,
            action_kind: Some(payload.kind.into()),
        },
        func.name.clone(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_create_action_func",
        serde_json::json!({
            "func_id": func.id,
            "func_name": func.name.to_owned(),
            "schema_variant_id": schema_variant_id,
        }),
    );

    ctx.commit().await?;

    Ok(Json(CreateVariantActionFuncV1Response { func_id: func.id }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantActionFuncV1Request {
    #[schema(value_type = String, example = "awsEC2InstanceCreate")]
    pub name: String,
    #[schema(value_type = Option<String>, example = "Create EC2 Instance")]
    pub display_name: Option<String>,
    #[schema(value_type = Option<String>, example = "Creates an EC2 Instance")]
    pub description: Option<String>,
    #[schema(value_type = String, example = "Create", pattern = "^(Create|Destroy|Manual|Refresh|Update)$")]
    pub kind: ActionKind,
    #[schema(value_type = String, example = "<!-- String escaped Typescript code here -->")]
    pub code: String,
    #[serde(default)]
    #[schema(value_type = Option<bool>, example = false)]
    pub skip_overlay: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantActionFuncV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub func_id: FuncId,
}

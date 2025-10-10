use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    FuncId,
    SchemaVariant,
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
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/management",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
        ("schema_variant_id" = String, Path, description = "Schema variant identifier"),
    ),
    summary = "Create a management function and attach to a schema variant",
    tag = "schemas",
    request_body = CreateVariantManagementFuncV1Request,
    responses(
        (status = 200, description = "Management function successfully created and attached to the variant", body = CreateVariantManagementFuncV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 412, description = "Schema variant not found for schema"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_variant_management(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SchemaVariantV1RequestPath {
        schema_id,
        schema_variant_id,
    }): Path<SchemaVariantV1RequestPath>,
    payload: Result<
        Json<CreateVariantManagementFuncV1Request>,
        axum::extract::rejection::JsonRejection,
    >,
) -> SchemaResult<Json<CreateVariantManagementFuncV1Response>> {
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

    let is_builtin = CachedModule::find_latest_for_schema_id(ctx, schema_id)
        .await?
        .is_some();
    let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;

    let func = if is_builtin {
        // Create an overlay function
        FuncAuthoringClient::create_new_management_func(ctx, Some(payload.name), schema_id).await?
    } else {
        if schema_variant.is_locked() {
            return Err(SchemaError::LockedVariant(schema_variant_id));
        }
        FuncAuthoringClient::create_new_management_func(ctx, Some(payload.name), schema_variant_id)
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

    let audit_sv_id = if is_builtin {
        None
    } else {
        Some(schema_variant_id)
    };

    let audit_schema_id = if is_builtin { Some(schema_id) } else { None };

    ctx.write_audit_log(
        AuditLogKind::AttachManagementFunc {
            func_id: func.id,
            func_display_name: func.display_name.clone(),
            schema_variant_id: audit_sv_id,
            schema_id: audit_schema_id,
            component_id: None,
            subject_name: schema_variant.display_name().to_string(),
        },
        func.name.clone(),
    )
    .await?;

    FuncAuthoringClient::publish_func_create_event(ctx, &func).await?;

    tracker.track(
        ctx,
        "api_create_management_func",
        serde_json::json!({
            "func_id": func.id,
            "func_name": func.name.to_owned(),
            "schema_variant_id": audit_sv_id,
            "schema_id": audit_schema_id,
            "overlay": is_builtin,
        }),
    );

    ctx.commit().await?;

    Ok(Json(CreateVariantManagementFuncV1Response {
        func_id: func.id,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantManagementFuncV1Request {
    #[schema(value_type = String, example = "awsCreateMyVpc")]
    pub name: String,
    #[schema(value_type = Option<String>, example = "Manage my VPC Components")]
    pub display_name: Option<String>,
    #[schema(value_type = Option<String>, example = "Manages a collection of VPC components and their relationships")]
    pub description: Option<String>,
    #[schema(value_type = String, example = "<!-- String escaped Typescript code here -->")]
    pub code: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantManagementFuncV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub func_id: FuncId,
}

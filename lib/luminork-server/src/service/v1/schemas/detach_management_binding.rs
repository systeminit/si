use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Func,
    SchemaVariant,
    func::binding::management::ManagementBinding,
    management::prototype::ManagementPrototype,
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
};

use super::{
    DetachFuncBindingV1Response,
    SchemaError,
    SchemaResult,
    SchemaVariantFuncV1RequestPath,
};

#[utoipa::path(
    delete,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/management/{func_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
        ("schema_variant_id" = String, Path, description = "Schema variant identifier"),
        ("func_id" = String, Path, description = "Func identifier"),
    ),
    summary = "Delete the binding between a management func and the schema variant",
    tag = "schemas",
    responses(
        (status = 200, description = "Management function successfully deteched from the variant", body = DetachFuncBindingV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 404, description = "Func not found"),
        (status = 412, description = "Schema variant not found for schema"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn detach_management_func_binding(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SchemaVariantFuncV1RequestPath {
        schema_id: _,
        schema_variant_id,
        func_id,
    }): Path<SchemaVariantFuncV1RequestPath>,
) -> SchemaResult<Json<DetachFuncBindingV1Response>> {
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(SchemaError::NotPermittedOnHead);
    }

    let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
    let func = Func::get_by_id(ctx, func_id).await?;

    // Get both variant-level AND schema-level (overlay) management prototypes
    let mgmt_prototypes =
        ManagementPrototype::list_for_schema_and_variant_id(ctx, schema_variant_id).await?;

    // Filter by func_id and delete matching prototypes
    for prototype in mgmt_prototypes {
        let prototype_func_id = ManagementPrototype::func_id(ctx, prototype.id()).await?;
        if prototype_func_id == func_id {
            ManagementBinding::delete_management_binding(ctx, prototype.id()).await?;

            tracker.track(
                ctx,
                "api_delete_management_func_binding",
                serde_json::json!({
                    "func_id": func.id,
                    "schema_variant_id": schema_variant_id,
                    "management_prototype_name": prototype.name(),
                }),
            );

            ctx.write_audit_log(
                AuditLogKind::DetachFunc {
                    func_id,
                    func_display_name: func.display_name.clone(),
                    schema_variant_id: Some(schema_variant_id),
                    schema_ids: None,
                    component_id: None,
                    subject_name: schema_variant.display_name().to_owned(),
                },
                func.name.clone(),
            )
            .await?;
        }
    }

    ctx.commit().await?;

    Ok(Json(DetachFuncBindingV1Response { success: true }))
}

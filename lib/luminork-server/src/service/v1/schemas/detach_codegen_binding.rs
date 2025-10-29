use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Func,
    SchemaVariant,
    func::{
        binding::{
            FuncBinding,
            leaf::LeafBinding,
        },
        leaf::LeafKind,
    },
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use si_events::audit_log::AuditLogKind;
use si_frontend_types::LeafBindingPrototype;
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
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/codegen/{func_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
        ("schema_variant_id" = String, Path, description = "Schema variant identifier"),
        ("func_id" = String, Path, description = "Func identifier"),
    ),
    summary = "Delete the binding between a codegen func and the schema variant",
    tag = "schemas",
    responses(
        (status = 200, description = "Codegen function successfully deteched from the variant", body = DetachFuncBindingV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 404, description = "Func not found"),
        (status = 412, description = "Schema variant not found for schema"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn detach_codegen_func_binding(
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

    let bindings =
        FuncBinding::get_bindings_for_schema_variant_id(ctx, func_id, schema_variant_id).await?;
    for binding in bindings {
        if let FuncBinding::CodeGeneration(codegen) = binding {
            if codegen.leaf_kind == LeafKind::CodeGeneration {
                match codegen.leaf_binding_prototype {
                    LeafBindingPrototype::Attribute(attribute_prototype_id) => {
                        LeafBinding::delete_leaf_func_binding(ctx, attribute_prototype_id).await?;
                    }
                    LeafBindingPrototype::Overlay(leaf_prototype_id) => {
                        LeafBinding::delete_leaf_overlay_func_binding(ctx, leaf_prototype_id)
                            .await?;
                    }
                }

                tracker.track(
                    ctx,
                    "api_delete_codegen_func_binding",
                    serde_json::json!({
                        "func_id": func.id,
                        "schema_variant_id": schema_variant_id,
                        "leaf_binding_prototype": codegen.leaf_binding_prototype,
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
    }

    ctx.commit().await?;

    Ok(Json(DetachFuncBindingV1Response { success: true }))
}

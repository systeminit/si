use axum::response::Json;
use dal::{
    ComponentId,
    FuncId,
    SchemaVariantId,
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

use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentsError,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/generate_template",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = GenerateTemplateV1Request,
    summary = "Generate a template",
    responses(
        (status = 200, description = "Template generated successfully", body = GenerateTemplateV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn generate_template(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<GenerateTemplateV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<GenerateTemplateV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    let (new_variant, _, func, prototype_id) = sdf_core::generate_template::prepare_and_generate(
        ctx,
        payload.component_ids,
        payload.asset_name.clone(),
        payload.func_name.clone(),
        payload.category.unwrap_or("Templates".to_string()),
        "#aaaaaa".to_string(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_generate_template",
        serde_json::json!({
            "generated_schema_variant_id": new_variant.id,
            "generated_prototype_id": prototype_id,
            "generated_func_id": func.id,
        }),
    );

    ctx.write_audit_log(
        AuditLogKind::GenerateTemplate {
            schema_variant_id: new_variant.id,
            management_prototype_id: prototype_id,
            func_id: func.id,
            func_name: payload.func_name,
            asset_name: payload.asset_name.to_owned(),
        },
        payload.asset_name,
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(GenerateTemplateV1Response {
        schema_variant_id: new_variant.id,
        func_id: func.id,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTemplateV1Request {
    #[schema(value_type = Vec<String>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79AA", "01H9ZQD35JPMBGHH69BT0Q79BB", "01H9ZQD35JPMBGHH69BT0Q79CC"]))]
    pub component_ids: Vec<ComponentId>,
    #[schema(example = "My Cool Template")]
    pub asset_name: String,
    #[schema(example = "Generate My Template")]
    pub func_name: String,
    #[schema(example = "Templates", required = false)]
    pub category: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTemplateV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79AA")]
    pub schema_variant_id: SchemaVariantId,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79BB")]
    pub func_id: FuncId,
}

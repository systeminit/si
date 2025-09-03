use axum::response::Json;
use dal::{
    SchemaVariant,
    schema::variant::authoring::VariantAuthoringClient,
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
    GetSchemaV1Response,
    SchemaError,
    SchemaResult,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "schemas",
    request_body = CreateSchemaV1Request,
    summary = "Create a schema and it's default variant",
    responses(
        (status = 200, description = "Schema created successfully", body = GetSchemaV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_schema(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<CreateSchemaV1Request>, axum::extract::rejection::JsonRejection>,
) -> SchemaResult<Json<GetSchemaV1Response>> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(SchemaError::NotPermittedOnHead);
    }

    let created_schema_variant = VariantAuthoringClient::create_schema_and_variant_from_code(
        ctx,
        payload.name.clone(),
        payload.description,
        payload.link,
        payload.category.unwrap_or("Custom".to_string()),
        payload.color.unwrap_or("#000000".to_string()),
        payload.code.clone(),
    )
    .await?;

    let schema = created_schema_variant.schema(ctx).await?;
    let variants = SchemaVariant::list_for_schema(ctx, schema.id()).await?;

    tracker.track(
        ctx,
        "api_create_schema",
        json!({
            "schema_id": schema.id(),
            "schema_variant_id": created_schema_variant.id,
            "display_name": created_schema_variant.display_name(),
            "category": created_schema_variant.category(),
        }),
    );

    ctx.write_audit_log(
        AuditLogKind::CreateSchemaVariant {
            schema_id: schema.id(),
            schema_variant_id: created_schema_variant.id,
        },
        created_schema_variant.display_name().to_string(),
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(GetSchemaV1Response {
        name: payload.name,
        default_variant_id: created_schema_variant.id,
        variant_ids: variants.into_iter().map(|v| v.id).collect(),
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchemaV1Request {
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub category: Option<String>,
    pub color: Option<String>,
    pub code: String,
}

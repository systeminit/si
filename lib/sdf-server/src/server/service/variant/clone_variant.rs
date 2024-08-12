use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::{SchemaVariantError, SchemaVariantResult};
use axum::extract::{Host, OriginalUri};
use axum::{response::IntoResponse, Json};
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, Schema, SchemaId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantRequest {
    pub id: SchemaId,
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantResponse {
    pub id: SchemaId,
    pub success: bool,
}

pub async fn clone_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CloneVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    if Schema::is_name_taken(&ctx, &request.name).await? {
        return Ok(axum::response::Response::builder()
            .status(409)
            .body("schema name already taken".to_string())?);
    }

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let schema = Schema::get_by_id(&ctx, request.id).await?;
    let default_schema_variant_id = schema.get_default_schema_variant_id(&ctx).await?.ok_or(
        SchemaVariantError::NoDefaultSchemaVariantFoundForSchema(schema.id()),
    )?;

    let (cloned_schema_variant, schema) = VariantAuthoringClient::new_schema_with_cloned_variant(
        &ctx,
        default_schema_variant_id,
        request.name.clone(),
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "clone_variant",
        serde_json::json!({
            "variant_name": request.name,
            "variant_category": cloned_schema_variant.category(),
            "variant_display_name": cloned_schema_variant.display_name(),
            "variant_id": cloned_schema_variant.id(),
            "variant_component_type": cloned_schema_variant.component_type(),
        }),
    );

    WsEvent::schema_variant_cloned(&ctx, cloned_schema_variant.id())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(serde_json::to_string(&CloneVariantResponse {
        id: schema.id(),
        success: true,
    })?)?)
}

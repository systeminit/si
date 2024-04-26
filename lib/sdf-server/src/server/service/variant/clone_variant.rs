use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::{SchemaVariantError, SchemaVariantResult};
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, Schema, SchemaId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantRequest {
    pub id: SchemaId,
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
    Json(request): Json<CloneVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let schema = Schema::get_by_id(&ctx, request.id).await?;
    if let Some(default_schema_variant_id) = schema.get_default_schema_variant(&ctx).await? {
        let (cloned_schema_variant, schema) =
            VariantAuthoringClient::clone_variant(&ctx, default_schema_variant_id).await?;

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "clone_variant",
            serde_json::json!({
                "variant_name": schema.name(),
                "variant_category": cloned_schema_variant.category(),
                "variant_menu_name": cloned_schema_variant.display_name(),
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
    } else {
        Err(SchemaVariantError::NoDefaultSchemaVariantFoundForSchema)
    }
}

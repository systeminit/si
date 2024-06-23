use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::{SchemaVariantError, SchemaVariantResult};
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, Schema, SchemaVariant, SchemaVariantId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantRequest {
    pub id: SchemaVariantId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantResponse {
    pub id: SchemaVariantId,
}

pub async fn create_unlocked_copy(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CloneVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let original_variant = SchemaVariant::get_by_id(&ctx, request.id).await?;

    let schema = original_variant.schema(&ctx).await?;

    if Schema::get_default_schema_variant_by_id(&ctx, schema.id()).await?
        != Some(original_variant.id())
    {
        return Err(SchemaVariantError::CreatingUnlockedCopyForNonDefault(
            original_variant.id(),
        ));
    }

    let unlocked_variant =
        VariantAuthoringClient::create_unlocked_variant_copy(&ctx, original_variant.id()).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_unlocked_variant_copy",
        serde_json::json!({
            "schema_name": schema.name(),
            "variant_version": original_variant.version(),
            "original_variant_id": original_variant.id(),
            "unlocked_variant_id": unlocked_variant.id(),
            "variant_component_type": original_variant.component_type(),
        }),
    );

    WsEvent::schema_variant_created(
        &ctx,
        unlocked_variant.schema(&ctx).await?.id(),
        unlocked_variant.id(),
        unlocked_variant.version().to_string(),
        unlocked_variant.category().to_string(),
        unlocked_variant.color().to_string(),
    )
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
        id: unlocked_variant.id(),
    })?)?)
}

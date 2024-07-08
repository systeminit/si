use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, WsEvent};
use dal::{SchemaVariantId, Visibility};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::SchemaVariantResult;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegenerateVariantRequest {
    // We need to get the updated data here, to ensure we create the prop the user is seeing
    pub variant: si_frontend_types::SchemaVariant,
    pub code: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegenerateVariantResponse {
    pub schema_variant_id: SchemaVariantId,
}

pub async fn regenerate_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(RegenerateVariantRequest {
        variant,
        code,
        visibility,
    }): Json<RegenerateVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let schema_variant_id = variant.schema_variant_id.into();

    VariantAuthoringClient::save_variant_content(
        &ctx,
        schema_variant_id,
        &variant.schema_name,
        &variant.display_name,
        &variant.category,
        variant.description,
        variant.link,
        &variant.color,
        variant.component_type.into(),
        code,
    )
    .await?;

    let updated_schema_variant_id =
        VariantAuthoringClient::regenerate_variant(&ctx, schema_variant_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "update_variant",
        serde_json::json!({
            "old_schema_variant_id": schema_variant_id,
            "new_schema_variant_id": updated_schema_variant_id,
        }),
    );

    WsEvent::schema_variant_update_finished(&ctx, schema_variant_id, updated_schema_variant_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(
        response.body(serde_json::to_string(&RegenerateVariantResponse {
            schema_variant_id: updated_schema_variant_id,
        })?)?,
    )
}

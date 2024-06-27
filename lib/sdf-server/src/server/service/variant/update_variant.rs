use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::Visibility;
use dal::{ChangeSet, SchemaVariantId, WsEvent};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::SchemaVariantResult;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVariantRequest {
    pub schema_variant_id: SchemaVariantId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

// TODO rename this to regenerate variant when the endpoint changes
pub async fn update_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(UpdateVariantRequest {
        schema_variant_id,
        visibility,
    }): Json<UpdateVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
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
    Ok(response.body(axum::body::Empty::new())?)
}

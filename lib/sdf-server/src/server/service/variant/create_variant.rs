use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, Schema, Visibility, WsEvent};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::SchemaVariantResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantRequest {
    pub name: String,
    pub color: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn create_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    if Schema::is_name_taken(&ctx, &request.name).await? {
        return Ok(axum::response::Response::builder()
            .status(409)
            .body("schema name already taken".to_string())?);
    }

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let created_schema_variant = VariantAuthoringClient::create_schema_and_variant(
        &ctx,
        request.name.clone(),
        None::<String>,
        None::<String>,
        "".to_string(),
        request.color.clone(),
    )
    .await?;

    let schema = created_schema_variant.schema(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_variant",
        serde_json::json!({
            "variant_name": request.name.clone(),
            "variant_id": created_schema_variant.id().clone(),
            "schema_id": schema.id(),
        }),
    );

    WsEvent::schema_variant_created(&ctx, schema.id(), created_schema_variant.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(serde_json::to_string(
        &created_schema_variant
            .into_frontend_type(&ctx, schema.id())
            .await?,
    )?)?)
}

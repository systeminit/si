use axum::{
    extract::{Host, OriginalUri},
    response::IntoResponse,
    Json,
};
use dal::{
    schema::variant::authoring::VariantAuthoringClient, ChangeSet, SchemaVariant, SchemaVariantId,
    Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::variant::SchemaVariantResult,
    track,
};

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
    Host(host_name): Host,
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
        &host_name,
        "update_variant",
        serde_json::json!({
            "old_schema_variant_id": schema_variant_id,
            "new_schema_variant_id": updated_schema_variant_id,
        }),
    );

    if schema_variant_id == updated_schema_variant_id {
        // if old == new -> send updated for it

        let schema =
            SchemaVariant::schema_id_for_schema_variant_id(&ctx, schema_variant_id).await?;
        let schema_variant = SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;

        WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    } else {
        // send that the old one is deleted and new one is created
        // (note: we auto upgrade components on regenerate now so this variant is actually eligible for GC)
        // let's pretend it was
        let schema =
            SchemaVariant::schema_id_for_schema_variant_id(&ctx, updated_schema_variant_id).await?;
        let schema_variant =
            SchemaVariant::get_by_id_or_error(&ctx, updated_schema_variant_id).await?;

        WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
            .await?
            .publish_on_commit(&ctx)
            .await?;
        WsEvent::schema_variant_deleted(&ctx, schema, schema_variant_id)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

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

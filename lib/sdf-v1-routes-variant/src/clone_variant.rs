use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{
    schema::variant::authoring::VariantAuthoringClient, ChangeSet, Schema, SchemaVariantId,
    Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_frontend_types::SchemaVariant as FrontendVariant;

use crate::{SchemaVariantError, SchemaVariantResult};
use sdf_core::{force_change_set_response::ForceChangeSetResponse, tracking::track};
use sdf_extract::{v1::AccessBuilder, HandlerContext, PosthogClient};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantRequest {
    pub id: SchemaVariantId,
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn clone_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CloneVariantRequest>,
) -> SchemaVariantResult<ForceChangeSetResponse<FrontendVariant>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;
    if Schema::is_name_taken(&ctx, &request.name).await? {
        return Err(SchemaVariantError::SchemaNameAlreadyTaken(request.name));
    }

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let (cloned_schema_variant, schema) = VariantAuthoringClient::new_schema_with_cloned_variant(
        &ctx,
        request.id,
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

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        cloned_schema_variant
            .into_frontend_type(&ctx, schema.id())
            .await?,
    ))
}

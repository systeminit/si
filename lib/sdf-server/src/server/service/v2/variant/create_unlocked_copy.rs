use axum::extract::{OriginalUri, Path};
use axum::response::IntoResponse;

use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, ChangeSetId, Schema, SchemaVariant, SchemaVariantId, WorkspacePk, WsEvent};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::{SchemaVariantError, SchemaVariantResult};

pub async fn create_unlocked_copy(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, schema_variant_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        SchemaVariantId,
    )>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let original_variant = SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;

    let schema = original_variant.schema(&ctx).await?;

    if Schema::get_default_schema_variant_by_id(&ctx, schema.id()).await?
        != Some(original_variant.id())
    {
        return Err(SchemaVariantError::CreatingUnlockedCopyForNonDefault(
            original_variant.id(),
        ));
    }

    if !original_variant.is_locked() {
        return Err(SchemaVariantError::VariantAlreadyUnlocked(
            original_variant.id,
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

    WsEvent::schema_variant_created(&ctx, schema.id(), unlocked_variant.clone())
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
        &unlocked_variant
            .into_frontend_type(&ctx, schema.id())
            .await?,
    )?)?)
}

use axum::extract::{Host, OriginalUri, Path};

use si_events::audit_log::AuditLogKind;
use si_frontend_types::SchemaVariant as FrontendVariant;

use dal::{
    schema::variant::authoring::VariantAuthoringClient, ChangeSet, ChangeSetId, Schema,
    SchemaVariant, SchemaVariantId, WorkspacePk, WsEvent,
};

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::v2::AccessBuilder,
    track,
};
use sdf_core::force_change_set_response::ForceChangeSetResponse;
use sdf_v1_routes_variant::{SchemaVariantError, SchemaVariantResult};

pub async fn create_unlocked_copy(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, schema_variant_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        SchemaVariantId,
    )>,
) -> SchemaVariantResult<ForceChangeSetResponse<FrontendVariant>> {
    let mut ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let original_variant = SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;

    let schema = original_variant.schema(&ctx).await?;

    if Schema::default_variant_id_opt(&ctx, schema.id()).await? != Some(original_variant.id()) {
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
    ctx.write_audit_log(
        AuditLogKind::UnlockSchemaVariant {
            schema_variant_id: unlocked_variant.id(),
            schema_variant_display_name: unlocked_variant.display_name().to_owned(),
        },
        schema.name().to_owned(),
    )
    .await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
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

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        unlocked_variant
            .into_frontend_type(&ctx, schema.id())
            .await?,
    ))
}

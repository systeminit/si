use axum::extract::{Host, OriginalUri, Path};
use dal::{ChangeSet, ChangeSetId, SchemaVariant, SchemaVariantId, WorkspacePk, WsEvent};
use si_events::audit_log::AuditLogKind;

use super::{SchemaVariantsAPIError, SchemaVariantsAPIResult};
use crate::{
    extract::{HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    service::v2::AccessBuilder,
    track,
};

pub async fn delete_unlocked_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, schema_variant_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        SchemaVariantId,
    )>,
) -> SchemaVariantsAPIResult<ForceChangeSetResponse<SchemaVariantId>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let schema_variant = SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;
    let schema = schema_variant.schema(&ctx).await?;

    let connected_components = SchemaVariant::list_component_ids(&ctx, schema_variant_id).await?;
    if !connected_components.is_empty() {
        return Err(SchemaVariantsAPIError::CannotDeleteVariantWithComponents);
    }

    SchemaVariant::cleanup_unlocked_variant(&ctx, schema_variant_id).await?;

    WsEvent::schema_variant_deleted(&ctx, schema.id(), schema_variant_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "delete_unlocked_variant",
        serde_json::json!({
            "schema_variant_id": schema_variant_id,
            "schema_variant_name": schema_variant.display_name(),
            "schema_variant_version": schema_variant.version(),
        }),
    );
    ctx.write_audit_log(
        AuditLogKind::DeleteSchemaVariant {
            schema_variant_id,
            schema_id: schema.id(),
        },
        schema_variant.display_name().to_owned(),
    )
    .await?;
    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        schema_variant_id,
    ))
}

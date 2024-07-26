use axum::{
    extract::{OriginalUri, Path},
    Json,
};

use dal::{ChangeSetId, SchemaVariant, SchemaVariantId, WorkspacePk};
use si_frontend_types as frontend_types;

use crate::{
    server::{
        extract::{AccessBuilder, HandlerContext, PosthogClient},
        tracking::track,
    },
    service::v2::variant::SchemaVariantsAPIError,
};

pub async fn get_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, schema_variant_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        SchemaVariantId,
    )>,
) -> Result<Json<frontend_types::SchemaVariant>, SchemaVariantsAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let schema_variant = SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;
    let schema_id = SchemaVariant::schema_id_for_schema_variant_id(&ctx, schema_variant_id).await?;
    let schema_variant = schema_variant.into_frontend_type(&ctx, schema_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_variant",
        serde_json::json!({
                    "schema_name": &schema_variant.schema_name,
                    "variant_category": &schema_variant.category,
                    "variant_menu_name": schema_variant.display_name,
                    "variant_id": schema_variant.schema_variant_id,
                    "schema_id": schema_variant.schema_id,
                    "variant_component_type": schema_variant.component_type,
        }),
    );

    Ok(Json(schema_variant))
}

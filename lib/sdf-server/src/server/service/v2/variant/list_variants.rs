use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};

use dal::{ChangeSetId, SchemaVariant, WorkspacePk};
use si_frontend_types as frontend_types;

use crate::{
    server::{
        extract::{AccessBuilder, HandlerContext, PosthogClient},
        tracking::track,
    },
    service::v2::variant::SchemaVariantsAPIError,
};

pub async fn list_variants(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<Vec<frontend_types::SchemaVariant>>, SchemaVariantsAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let schema_variants = SchemaVariant::list_user_facing(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "list_variants",
        serde_json::json!({}),
    );

    Ok(Json(schema_variants))
}

use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{cached_module::CachedModule, ChangeSetId, SchemaVariant, WorkspacePk};
use frontend_types::{SchemaVariant as FrontendVariant, UninstalledVariant};
use serde::{Deserialize, Serialize};
use si_frontend_types as frontend_types;

use crate::SchemaVariantsAPIError;
use axum_util::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListVariantsResponse {
    installed: Vec<FrontendVariant>,
    uninstalled: Vec<UninstalledVariant>,
}

pub async fn list_variants(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<ListVariantsResponse>, SchemaVariantsAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let installed = SchemaVariant::list_user_facing(&ctx).await?;
    let uninstalled: Vec<UninstalledVariant> = CachedModule::latest_modules(&ctx)
        .await?
        .into_iter()
        .filter(|module| {
            !installed
                .iter()
                .any(|variant| variant.schema_id == module.schema_id.into())
        })
        .map(Into::into)
        .collect();

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "list_variants",
        serde_json::json!({}),
    );

    Ok(Json(ListVariantsResponse {
        installed,
        uninstalled,
    }))
}

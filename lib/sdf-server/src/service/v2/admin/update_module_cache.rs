use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use axum::{
    extract::{Host, OriginalUri},
    response::Json,
};
use dal::cached_module::CachedModule;

use super::AdminAPIResult;
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateModuleCacheResponse {
    new_modules: Vec<CachedModule>,
}

#[instrument(name = "admin.update_module_cache", skip_all)]
pub async fn update_module_cache(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
) -> AdminAPIResult<Json<UpdateModuleCacheResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let new_modules = CachedModule::update_cached_modules(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "update_module_cache",
        serde_json::json!({ "new_modules": new_modules }),
    );

    Ok(Json(UpdateModuleCacheResponse { new_modules }))
}

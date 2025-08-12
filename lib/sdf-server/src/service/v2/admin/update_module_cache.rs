use axum::{
    extract::{
        Host,
        OriginalUri,
    },
    http::Uri,
    response::Json,
};
use dal::{
    DalContext,
    WsEvent,
    cached_module::CachedModule,
};
use sdf_core::async_route::handle_error;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Tenancy;
use telemetry::prelude::*;
use ulid::Ulid;

use super::{
    AdminAPIResult,
    AdminUserContext,
};
use crate::{
    extract::{
        PosthogClient,
        workspace::TargetWorkspaceIdFromToken,
    },
    track,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateModuleCacheResponse {
    pub id: Ulid,
}

#[instrument(name = "admin.update_module_cache", skip_all)]
pub async fn update_module_cache(
    workspace_id: TargetWorkspaceIdFromToken,
    AdminUserContext(mut ctx): AdminUserContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
) -> AdminAPIResult<Json<UpdateModuleCacheResponse>> {
    let task_id = Ulid::new();

    ctx.update_tenancy(Tenancy::new(workspace_id.into()));
    tokio::task::spawn(async move {
        if let Err(err) = update_cached_modules_inner(
            &ctx,
            &original_uri,
            &host_name,
            PosthogClient(posthog_client),
        )
        .await
        {
            return handle_error(&ctx, original_uri, task_id, err).await;
        };

        let event = match WsEvent::async_finish_workspace(&ctx, task_id).await {
            Ok(event) => event,
            Err(err) => {
                return handle_error(&ctx, original_uri, task_id, err).await;
            }
        };

        if let Err(err) = event.publish_immediately(&ctx).await {
            handle_error(&ctx, original_uri, task_id, err).await;
        };
    });

    Ok(Json(UpdateModuleCacheResponse { id: task_id }))
}

pub async fn update_cached_modules_inner(
    ctx: &DalContext,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): PosthogClient,
) -> AdminAPIResult<()> {
    info!("Starting module cache update");
    CachedModule::update_cached_modules(ctx).await?;

    track(
        &posthog_client,
        ctx,
        original_uri,
        host_name,
        "update_module_cache",
        serde_json::json!({}),
    );

    Ok(())
}

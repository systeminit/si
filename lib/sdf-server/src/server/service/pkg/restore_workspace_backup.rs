use super::PkgResult;
use crate::server::extract::RawAccessToken;
use crate::server::tracking::track;
use crate::{
    server::extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::pkg::PkgError,
};
use axum::extract::OriginalUri;
use axum::Json;
use module_index_client::IndexClient;
use serde::{Deserialize, Serialize};
use si_pkg::SiPkg;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreBackupRequest {
    pub id: Ulid,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreBackupResponse {
    pub success: bool,
}

pub async fn restore_workspace_backup(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<RestoreBackupRequest>,
) -> PkgResult<Json<RestoreBackupResponse>> {
    let ctx = builder.build_head(request_ctx).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(PkgError::ModuleIndexNotConfigured),
    };

    let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let pkg_data = module_index_client.download_module(request.id).await?;

    let _pkg = SiPkg::load_from_bytes(pkg_data)?;

    // TODO: call restore from package function

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "restore_workspace_backup",
        serde_json::json!({}),
    );

    // TODO: new realtime event that will trigger a pop-up and force user to reload?
    // WsEvent::change_set_written(&ctx)
    //     .await?
    //     .publish_on_commit(&ctx)
    //     .await?;
    ctx.commit().await?;

    Ok(Json(RestoreBackupResponse { success: true }))
}

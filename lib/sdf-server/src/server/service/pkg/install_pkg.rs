use super::PkgResult;
use crate::server::extract::RawAccessToken;
use crate::server::tracking::track;
use crate::{
    server::extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::pkg::PkgError,
};
use axum::extract::OriginalUri;
use axum::Json;
use dal::{
    pkg::{import_pkg_from_pkg, ImportSkips},
    Visibility, WsEvent,
};
use module_index_client::IndexClient;
use serde::{Deserialize, Serialize};
use si_pkg::SiPkg;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallPkgRequest {
    pub id: Ulid,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallPkgResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_skips: Option<Vec<ImportSkips>>,
}

pub async fn install_pkg(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<InstallPkgRequest>,
) -> PkgResult<Json<InstallPkgResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(PkgError::ModuleIndexNotConfigured),
    };

    let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let pkg_data = module_index_client.download_module(request.id).await?;

    let pkg = SiPkg::load_from_bytes(pkg_data)?;
    let pkg_name = pkg.metadata()?.name().to_owned();
    let (_, _, import_skips) = import_pkg_from_pkg(&ctx, &pkg, None).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "install_pkg",
        serde_json::json!({
                    "pkg_name": pkg_name,
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(InstallPkgResponse {
        success: true,
        import_skips,
    }))
}

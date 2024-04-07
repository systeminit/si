use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::module::Module;
use dal::Visibility;
use serde::{Deserialize, Serialize};

use super::PkgResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgListRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgListResponse {
    pub pkgs: Vec<PkgView>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgView {
    name: String,
    hash: String,
}

pub async fn list_pkgs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Query(request): Query<PkgListRequest>,
) -> PkgResult<Json<PkgListResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let installed_pkgs = Module::list_installed(&ctx).await?;

    let pkgs: Vec<PkgView> = installed_pkgs
        .iter()
        .map(|pkg| PkgView {
            name: pkg.name().to_owned(),
            hash: pkg.root_hash().to_string(),
        })
        .collect();

    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "list_pkgs",
    //     serde_json::json!({}),
    // );

    Ok(Json(PkgListResponse { pkgs }))
}

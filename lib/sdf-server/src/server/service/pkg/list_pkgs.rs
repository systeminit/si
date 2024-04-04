use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use dal::Visibility;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};

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
    HandlerContext(_builder): HandlerContext,
    AccessBuilder(_request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Query(_request): Query<PkgListRequest>,
) -> PkgResult<Json<PkgListResponse>> {
    // let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    //
    // let installed_pkgs = InstalledPkg::list(&ctx).await?;
    //
    // let pkgs: Vec<PkgView> = installed_pkgs
    //     .iter()
    //     .map(|pkg| PkgView {
    //         name: pkg.name().to_owned(),
    //         hash: pkg.root_hash().to_string(),
    //     })
    //     .collect();
    //
    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "list_pkgs",
    //     serde_json::json!({}),
    // );
    //
    // Ok(Json(PkgListResponse { pkgs }))

    Ok(Json(PkgListResponse { pkgs: vec![] }))
}

use super::{get_pkgs_path, list_pkg_dir_entries, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::{installed_pkg::InstalledPkg, StandardModel, Visibility};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    installed: bool,
    hash: Option<String>,
}

#[remain::sorted]
enum PackageMapEntry {
    InstalledPkg(InstalledPkg),
    UninstalledPkg,
}

pub async fn list_pkgs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<PkgListRequest>,
) -> PkgResult<Json<PkgListResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let pkgs_path = get_pkgs_path(&builder).await?;

    let installed_pkgs = InstalledPkg::list(&ctx).await?;

    let mut installed_pkg_map: HashMap<String, PackageMapEntry> = HashMap::new();
    for installed_pkg in installed_pkgs {
        installed_pkg_map.insert(
            installed_pkg.name().to_string(),
            PackageMapEntry::InstalledPkg(installed_pkg),
        );
    }

    for available_pkg in list_pkg_dir_entries(pkgs_path).await? {
        installed_pkg_map
            .entry(available_pkg)
            .or_insert(PackageMapEntry::UninstalledPkg);
    }

    let pkgs: Vec<PkgView> = installed_pkg_map
        .drain()
        .map(|(name, pkg)| match pkg {
            PackageMapEntry::UninstalledPkg => PkgView {
                name,
                installed: false,
                hash: None,
            },
            PackageMapEntry::InstalledPkg(installed_pkg) => PkgView {
                name,
                installed: true,
                hash: Some(installed_pkg.root_hash().to_string()),
            },
        })
        .collect();

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "list_pkgs",
        serde_json::json!({}),
    );

    Ok(Json(PkgListResponse { pkgs }))
}

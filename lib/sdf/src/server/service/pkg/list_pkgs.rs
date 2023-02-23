use super::{get_pkgs_path, list_pkg_dir_entries, PkgResult, PkgView};
use crate::server::extract::{AccessBuilder, HandlerContext};
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

pub async fn list_pkgs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<PkgListRequest>,
) -> PkgResult<Json<PkgListResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let pkgs_path = get_pkgs_path(&builder).await?;

    let installed_pkgs = InstalledPkg::list(&ctx).await?;

    let mut installed_pkg_map: HashMap<String, bool> = HashMap::new();
    for installed_pkg in installed_pkgs {
        installed_pkg_map.insert(installed_pkg.name().to_string(), true);
    }

    for available_pkg in list_pkg_dir_entries(pkgs_path).await? {
        installed_pkg_map.entry(available_pkg).or_insert(false);
    }

    let pkgs: Vec<PkgView> = installed_pkg_map
        .drain()
        .map(|(name, installed)| PkgView { name, installed })
        .collect();

    Ok(Json(PkgListResponse { pkgs }))
}

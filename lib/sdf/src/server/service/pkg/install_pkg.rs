use super::{get_pkgs_path, pkg_lookup, PkgError, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{installed_pkg::InstalledPkg, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallPkgRequest {
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallPkgResponse {
    pub success: bool,
}

pub async fn install_pkg(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<InstallPkgRequest>,
) -> PkgResult<Json<InstallPkgResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let real_pkg_file_name = pkg_lookup(get_pkgs_path(&builder).await?, &request.name).await?;

    let real_pkg_file_name = match real_pkg_file_name {
        None => return Err(PkgError::PackageNotFound(request.name)),
        Some(real_pkg_file_name) => real_pkg_file_name,
    };

    if !InstalledPkg::find_by_attr(&ctx, "name", &real_pkg_file_name)
        .await?
        .is_empty()
    {
        return Err(PkgError::PackageAlreadyInstalled(real_pkg_file_name));
    }

    InstalledPkg::new(&ctx, &request.name, None, None).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(InstallPkgResponse { success: true }))
}

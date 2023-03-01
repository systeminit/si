use super::{get_pkgs_path, pkg_lookup, PkgError, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{installed_pkg::InstalledPkg, pkg::import_pkg, StandardModel, Visibility, WsEvent};
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

    let real_pkg_path = pkg_lookup(get_pkgs_path(&builder).await?, &request.name).await?;

    let (real_pkg_file_name, real_pkg_path) = match real_pkg_path {
        None => return Err(PkgError::PackageNotFound(request.name)),
        Some(real_pkg_path) => match real_pkg_path.file_name() {
            None => unreachable!(),
            Some(file_name) => (file_name.to_string_lossy().to_string(), real_pkg_path),
        },
    };

    if !InstalledPkg::find_by_attr(&ctx, "name", &real_pkg_file_name)
        .await?
        .is_empty()
    {
        return Err(PkgError::PackageAlreadyInstalled(real_pkg_file_name));
    }

    let pkg = import_pkg(&ctx, &real_pkg_path).await?;
    InstalledPkg::new(&ctx, &request.name, Some(pkg.hash()?.to_string()), None).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(InstallPkgResponse { success: true }))
}

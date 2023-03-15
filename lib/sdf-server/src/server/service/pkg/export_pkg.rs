use super::{get_new_pkg_path, PkgError, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{SchemaVariantId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportPkgRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub schema_variants: Vec<SchemaVariantId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportPkgResponse {
    pub success: bool,
    pub full_path: String,
}

pub async fn export_pkg(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<ExportPkgRequest>,
) -> PkgResult<Json<ExportPkgResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    if request.name.trim().is_empty() {
        return Err(PkgError::PackageNameEmpty);
    }

    if request.version.trim().is_empty() {
        return Err(PkgError::PackageVersionEmpty);
    }

    if request.schema_variants.is_empty() {
        return Err(PkgError::PackageExportEmpty);
    }

    let new_pkg_path = get_new_pkg_path(&builder, &request.name, &request.version).await?;

    dal::pkg::export_pkg(
        &ctx,
        &new_pkg_path,
        request.name,
        request.version,
        request.description,
        "Sally Signup".to_string(),
        request.schema_variants,
    )
    .await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(ExportPkgResponse {
        success: true,
        full_path: new_pkg_path.to_string_lossy().to_string(),
    }))
}

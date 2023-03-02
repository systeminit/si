use super::{pkg_open, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use chrono::{DateTime, Utc};
use dal::{installed_pkg::InstalledPkg, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgGetRequest {
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgGetResponse {
    pub name: String,
    pub hash: String,
    pub version: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub schemas: Vec<String>,
    pub installed: bool,
}

pub async fn get_pkg(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<PkgGetRequest>,
) -> PkgResult<Json<PkgGetResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let pkg = pkg_open(&builder, &request.name).await?;

    let schemas = pkg
        .schemas()?
        .iter()
        .map(|schema| schema.name().to_string())
        .collect();

    let metadata = pkg.metadata()?;
    let root_hash = pkg.hash()?.to_string();
    let installed = !InstalledPkg::find_by_attr(&ctx, "root_hash", &root_hash)
        .await?
        .is_empty();

    Ok(Json(PkgGetResponse {
        hash: root_hash,
        name: metadata.name().to_string(),
        version: metadata.version().to_string(),
        description: metadata.description().to_string(),
        created_at: metadata.created_at(),
        created_by: metadata.created_by().to_string(),
        installed,
        schemas,
    }))
}

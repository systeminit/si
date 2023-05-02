use axum::{extract::Query, Json};
use chrono::{DateTime, Utc};
use dal::{installed_pkg::InstalledPkg, StandardModel, Visibility};
use serde::{Deserialize, Serialize};
use std::cmp::{Ord, PartialOrd};

use super::{pkg_open, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use si_pkg::PkgSpec;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgGetRequest {
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PkgFuncView {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

impl Ord for PkgFuncView {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for PkgFuncView {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
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
    pub funcs: Vec<PkgFuncView>,
    pub spec: serde_json::Value,
    pub installed: bool,
}

pub async fn get_pkg(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<PkgGetRequest>,
) -> PkgResult<Json<PkgGetResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let pkg = pkg_open(&builder, &request.name).await?;

    let mut schemas: Vec<String> = pkg
        .schemas()?
        .iter()
        .map(|schema| schema.name().to_string())
        .collect();
    schemas.sort();

    let mut funcs: Vec<PkgFuncView> = pkg
        .funcs()?
        .iter()
        .map(|func| PkgFuncView {
            name: func.name().to_string(),
            display_name: func.display_name().map(|dname| dname.to_string()),
            description: func.description().map(|desc| desc.to_string()),
        })
        .collect();
    funcs.sort();

    let metadata = pkg.metadata()?;
    let root_hash = pkg.hash()?.to_string();
    let installed = !InstalledPkg::find_by_attr(&ctx, "root_hash", &root_hash)
        .await?
        .is_empty();

    // This type can be serialized to json with serde_json::to_string/to_string_pretty
    let pkg_spec = PkgSpec::try_from(pkg)?;

    Ok(Json(PkgGetResponse {
        hash: root_hash,
        name: metadata.name().to_string(),
        version: metadata.version().to_string(),
        description: metadata.description().to_string(),
        created_at: metadata.created_at(),
        created_by: metadata.created_by().to_string(),
        spec: serde_json::to_value(&pkg_spec)?,
        installed,
        schemas,
        funcs,
    }))
}

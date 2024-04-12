use std::cmp::{Ord, PartialOrd};

use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use dal::module::Module;
use dal::Visibility;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};

use super::{ModuleError, ModuleResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgGetRequest {
    pub hash: String,
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
    pub installed: bool,
}

pub async fn get_module_by_hash(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Query(request): Query<PkgGetRequest>,
) -> ModuleResult<Json<PkgGetResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let installed_pkg = match Module::find_by_root_hash(&ctx, &request.hash).await? {
        Some(m) => m,
        None => return Err(ModuleError::ModuleHashNotFound(request.hash.to_string())),
    };

    let mut pkg_schemas: Vec<String> = installed_pkg
        .list_associated_schemas(&ctx)
        .await?
        .iter()
        .map(|s| s.name.clone())
        .collect();
    pkg_schemas.sort();

    let mut pkg_funcs: Vec<PkgFuncView> = installed_pkg
        .list_associated_funcs(&ctx)
        .await?
        .iter()
        .map(|f| PkgFuncView {
            name: f.clone().name.to_string(),
            display_name: f.clone().display_name,
            description: f.clone().description,
        })
        .collect();
    pkg_funcs.sort();

    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "get_pkg",
    //     serde_json::json!({
    //                 "pkg_name": metadata.clone().name(),
    //                 "pkg_version": metadata.clone().version(),
    //                 "pkg_schema_count": schemas.len(),
    //                 "pkg_funcs_count":  funcs.len(),
    //                 "pkg_is_installed":  installed.clone(),
    //     }),
    // );

    Ok(Json(PkgGetResponse {
        hash: installed_pkg.root_hash().to_string(),
        name: installed_pkg.name().to_string(),
        version: installed_pkg.version().to_string(),
        description: installed_pkg.description().to_string(),
        created_at: installed_pkg.created_at(),
        created_by: installed_pkg.created_by_email().to_string(),
        installed: true,
        schemas: pkg_schemas,
        funcs: pkg_funcs,
    }))
}

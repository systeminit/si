use axum::{
    Json,
    extract::{Host, OriginalUri, Path, Query},
};
use chrono::{DateTime, Utc};
use dal::{ChangeSetId, WorkspacePk, module::Module};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::v2::AccessBuilder,
    track,
};

use super::{ModuleAPIResult, ModulesAPIError};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetModuleByHashRequest {
    pub hash: String,
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
pub struct GetModuleByHashResponse {
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

pub async fn module_by_hash(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Query(request): Query<GetModuleByHashRequest>,
) -> ModuleAPIResult<Json<GetModuleByHashResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let installed_pkg = match Module::find_by_root_hash(&ctx, &request.hash).await? {
        Some(m) => m,
        None => {
            return Err(ModulesAPIError::ModuleHashNotFound(
                request.hash.to_string(),
            ));
        }
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

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "get_module",
        serde_json::json!({
            "pkg_name": installed_pkg.clone().name(),
            "pkg_version": installed_pkg.clone().version(),
            "pkg_schema_count": pkg_schemas.len(),
            "pkg_funcs_count":  pkg_funcs.len(),
        }),
    );

    Ok(Json(GetModuleByHashResponse {
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

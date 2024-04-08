use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::module::Module;
use dal::Visibility;
use serde::{Deserialize, Serialize};

use super::ModuleResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModuleListRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModuleListResponse {
    pub modules: Vec<ModuleView>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModuleView {
    name: String,
    hash: String,
}

pub async fn list_modules(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Query(request): Query<ModuleListRequest>,
) -> ModuleResult<Json<ModuleListResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let installed_modules = Module::list_installed(&ctx).await?;

    let modules: Vec<ModuleView> = installed_modules
        .iter()
        .map(|module| ModuleView {
            name: module.name().to_owned(),
            hash: module.root_hash().to_string(),
        })
        .collect();

    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "list_pkgs",
    //     serde_json::json!({}),
    // );

    Ok(Json(ModuleListResponse { modules }))
}

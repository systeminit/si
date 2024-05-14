use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};

use dal::module::Module;
use dal::Visibility;

use crate::server::extract::{AccessBuilder, HandlerContext};

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

    Ok(Json(ModuleListResponse { modules }))
}

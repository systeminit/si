use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{module::Module, ChangeSetId, WorkspacePk};
use si_frontend_types::ModuleSummary;

use super::ModuleAPIResult;
use crate::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};

pub async fn list(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(_raw_access_token): RawAccessToken,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> ModuleAPIResult<Json<Vec<ModuleSummary>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let installed_modules = Module::list_installed(&ctx).await?;

    let modules: Vec<ModuleSummary> = installed_modules
        .iter()
        .map(|module| ModuleSummary {
            name: module.name().to_owned(),
            hash: module.root_hash().to_string(),
        })
        .collect();

    Ok(Json(modules))
}

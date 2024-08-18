use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{module::Module, ChangeSetId, WorkspacePk};
use module_index_client::ModuleIndexClient;
use si_frontend_types as frontend_types;

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken},
    tracking::track,
};

use super::ModulesAPIError;

pub async fn sync(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<frontend_types::SyncedModules>, ModulesAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let (latest_modules, module_details) = {
        let module_index_url = ctx
            .module_index_url()
            .ok_or(ModulesAPIError::ModuleIndexNotConfigured)?;
        let module_index_client =
            ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token);
        (
            module_index_client.list_latest_modules().await?,
            module_index_client.list_builtins().await?,
        )
    };

    let synced_modules = Module::sync(&ctx, latest_modules.modules, module_details.modules).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "sync",
        serde_json::json!({}),
    );

    Ok(Json(synced_modules))
}

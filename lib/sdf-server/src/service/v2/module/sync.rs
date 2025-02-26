use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{module::Module, ChangeSetId, WorkspacePk};
use si_frontend_types as frontend_types;

use super::ModulesAPIError;
use crate::{
    extract::{HandlerContext, PosthogClient},
    service::v2::AccessBuilder,
    track,
};

pub async fn sync(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<frontend_types::SyncedModules>, ModulesAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // TODO(nick): the concept of the inner types returned here are from the module index days and
    // are bullshit. We should provide the minimal set of information, which is moreso just telling
    // the frontend which _schemas_ are ready to be upgraded or contributed. Yes, schema variants
    // are the important granular bits, but in the case of upgradeables, you find the cached module
    // that is "latest" and in the case of contributeables, you can only contribute the default
    // variant.
    let synced_modules = Module::sync(&ctx).await?;

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

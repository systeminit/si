use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::WsEvent;
use module_index_client::ModuleIndexClient;
use sdf_core::tracking::track;
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    request::RawAccessToken,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::{
    HistoryActor,
    User,
};
use ulid::Ulid;

use crate::{
    ModuleError,
    ModuleResult,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BeginImportFlow {
    pub id: Ulid,
}

pub async fn begin_approval_process(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    Json(request): Json<BeginImportFlow>,
) -> ModuleResult<Json<()>> {
    let ctx = builder.build_head(request_ctx).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModuleError::ModuleIndexNotConfigured),
    };

    let module_index_client =
        ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token)?;
    let pkg_data = module_index_client.download_workspace(request.id).await?;

    let metadata = pkg_data.into_latest().metadata;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk).await?,

        HistoryActor::SystemInit => {
            return Err(ModuleError::InvalidUserSystemInit);
        }
    };

    let workspace_pk = ctx
        .tenancy()
        .workspace_pk_opt()
        .ok_or(ModuleError::ExportingImportingWithRootTenancy)?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "begin_approval_process",
        serde_json::json!({
            "how": "/pkg/begin_approval_process",
            "workspace_pk": workspace_pk,
        }),
    );

    WsEvent::workspace_import_begin_approval_process(
        &ctx,
        Some(workspace_pk),
        Some(user.pk()),
        metadata.created_at,
        metadata.created_by,
        metadata.name,
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    WsEvent::import_workspace_vote(&ctx, Some(workspace_pk), user.pk(), "Approve".to_string())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(()))
}

pub async fn cancel_approval_process(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
) -> ModuleResult<Json<()>> {
    let ctx = builder.build_head(request_ctx).await?;

    let user_pk = match ctx.history_actor() {
        HistoryActor::User(user_pk) => {
            let user = User::get_by_pk(&ctx, *user_pk).await?;

            Some(user.pk())
        }

        HistoryActor::SystemInit => None,
    };

    let workspace_pk = ctx
        .tenancy()
        .workspace_pk_opt()
        .ok_or(ModuleError::ExportingImportingWithRootTenancy)?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "cancel_approval_process",
        serde_json::json!({
            "how": "/pkg/cancel_approval_process",
            "workspace_pk": workspace_pk,
        }),
    );

    WsEvent::workspace_import_cancel_approval_process(&ctx, Some(workspace_pk), user_pk)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(()))
}

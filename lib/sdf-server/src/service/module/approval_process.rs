use anyhow::Result;
use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{HistoryActor, User, WsEvent};
use module_index_client::ModuleIndexClient;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    extract::{request::RawAccessToken, v1::AccessBuilder, HandlerContext, PosthogClient},
    service::module::ModuleError,
    track,
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
) -> Result<Json<()>> {
    let ctx = builder.build_head(request_ctx).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModuleError::ModuleIndexNotConfigured.into()),
    };

    let module_index_client =
        ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let pkg_data = module_index_client.download_workspace(request.id).await?;

    let metadata = pkg_data.into_latest().metadata;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk)
            .await?
            .ok_or(ModuleError::InvalidUser(*user_pk))?,

        HistoryActor::SystemInit => {
            return Err(ModuleError::InvalidUserSystemInit.into());
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
) -> Result<Json<()>> {
    let ctx = builder.build_head(request_ctx).await?;

    let user_pk = match ctx.history_actor() {
        HistoryActor::User(user_pk) => {
            let user = User::get_by_pk(&ctx, *user_pk)
                .await?
                .ok_or(ModuleError::InvalidUser(*user_pk))?;

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

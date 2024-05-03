use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};
use crate::server::tracking::track;
use crate::service::module::{ModuleError, ModuleResult};
use axum::extract::OriginalUri;
use axum::Json;
use dal::{HistoryActor, User, WsEvent};
use module_index_client::IndexClient;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BeginImportFlow {
    pub id: Ulid,
}

pub async fn begin_approval_process(
    OriginalUri(original_uri): OriginalUri,
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

    let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let pkg_data = module_index_client.download_workspace(request.id).await?;

    let metadata = pkg_data.into_latest().metadata;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk)
            .await?
            .ok_or(ModuleError::InvalidUser(*user_pk))?,

        HistoryActor::SystemInit => {
            return Err(ModuleError::InvalidUserSystemInit);
        }
    };

    let workspace_pk = ctx
        .tenancy()
        .workspace_pk()
        .ok_or(ModuleError::ExportingImportingWithRootTenancy)?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
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
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
) -> ModuleResult<Json<()>> {
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
        .workspace_pk()
        .ok_or(ModuleError::ExportingImportingWithRootTenancy)?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
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

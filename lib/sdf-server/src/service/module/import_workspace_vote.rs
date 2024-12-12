use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{HistoryActor, User, WsEvent};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    routes::AppError,
    service::module::ModuleError,
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportVoteRequest {
    pub vote: String,
}

pub async fn import_workspace_vote(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<ImportVoteRequest>,
) -> Result<Json<()>, AppError> {
    let ctx = builder.build_head(request_ctx).await?;

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
        "merge_vote",
        serde_json::json!({
            "how": "/variant_definition/import_vote",
            "workspace_pk": workspace_pk,
            "user_pk": user.pk(),
            "vote": request.vote,
        }),
    );

    WsEvent::import_workspace_vote(&ctx, Some(workspace_pk), user.pk(), request.vote)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(()))
}

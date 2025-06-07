use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSetId,
    WorkspacePk,
    module::ModuleId,
};
use module_index_client::ModuleIndexClient;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::{
    HistoryActor,
    User,
};

use super::{
    ModuleAPIResult,
    ModulesAPIError,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
        request::RawAccessToken,
    },
    service::v2::AccessBuilder,
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PromoteToBuiltinModuleResponse {
    pub success: bool,
}

pub async fn promote(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, module_id)): Path<(WorkspacePk, ChangeSetId, ModuleId)>,
) -> ModuleAPIResult<Json<PromoteToBuiltinModuleResponse>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModulesAPIError::ModuleIndexNotConfigured),
    };

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk_opt(&ctx, *user_pk).await?,
        _ => None,
    };

    let (_, created_by_email) = user
        .map(|user| (user.name().to_owned(), user.email().to_owned()))
        .unwrap_or((
            "unauthenticated user name".into(),
            "unauthenticated user email".into(),
        ));

    let module_index_client =
        ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token)?;

    module_index_client
        .promote_to_builtin(module_id.into(), created_by_email.clone())
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "promote_to_builtin",
        serde_json::json!({
                    "pkg_id": module_id,
                    "pkg_promoted_to_builtin_by": created_by_email,
        }),
    );

    ctx.commit().await?;

    Ok(Json(PromoteToBuiltinModuleResponse { success: true }))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RejectModuleResponse {
    pub success: bool,
}

pub async fn reject(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, module_id)): Path<(WorkspacePk, ChangeSetId, ModuleId)>,
) -> ModuleAPIResult<Json<RejectModuleResponse>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModulesAPIError::ModuleIndexNotConfigured),
    };

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk_opt(&ctx, *user_pk).await?,
        _ => None,
    };

    let (_, created_by_email) = user
        .map(|user| (user.name().to_owned(), user.email().to_owned()))
        .unwrap_or((
            "unauthenticated user name".into(),
            "unauthenticated user email".into(),
        ));

    let module_index_client =
        ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token)?;

    module_index_client
        .reject_module(module_id.into(), created_by_email.clone())
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "reject_pkg",
        serde_json::json!({
                    "pkg_id": module_id,
                    "pkg_rejected_by": created_by_email,
        }),
    );

    ctx.commit().await?;

    Ok(Json(RejectModuleResponse { success: true }))
}

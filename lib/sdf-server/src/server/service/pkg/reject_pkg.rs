use super::PkgResult;
use crate::server::extract::RawAccessToken;
use crate::server::tracking::track;
use crate::{
    server::extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::pkg::PkgError,
};
use axum::extract::OriginalUri;
use axum::Json;
use dal::{HistoryActor, User, Visibility, WsEvent};
use module_index_client::IndexClient;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RejectPkgRequest {
    pub id: Ulid,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RejectPkgResponse {
    pub success: bool,
}

pub async fn reject_pkg(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<RejectPkgRequest>,
) -> PkgResult<Json<RejectPkgResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(PkgError::ModuleIndexNotConfigured),
    };

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk).await?,
        _ => None,
    };

    let (_, created_by_email) = user
        .map(|user| (user.name().to_owned(), user.email().to_owned()))
        .unwrap_or((
            "unauthenticated user name".into(),
            "unauthenticated user email".into(),
        ));

    let module_id = request.id;

    let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);

    module_index_client
        .reject_module(module_id, created_by_email.clone())
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "reject_pkg",
        serde_json::json!({
                    "pkg_id": module_id,
                    "pkg_rejected_by": created_by_email,
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(RejectPkgResponse { success: true }))
}

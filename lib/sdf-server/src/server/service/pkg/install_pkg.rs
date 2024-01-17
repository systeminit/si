use std::str::FromStr;

use super::PkgResult;
use crate::server::extract::RawAccessToken;
use crate::server::tracking::track;
use crate::{
    server::extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::pkg::PkgError,
};
use axum::extract::OriginalUri;
use axum::http::uri::Uri;
use axum::{response::IntoResponse, Json};
use dal::{pkg::import_pkg_from_pkg, ChangeSet, Visibility, WsEvent};
use dal::{DalContext, HistoryActor, User, WorkspacePk};
use module_index_client::IndexClient;
use serde::{Deserialize, Serialize};
use si_pkg::{SiPkg, SiPkgKind};
use telemetry::prelude::*;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallPkgRequest {
    pub id: Ulid,
    pub override_builtin_schema_feature_flag: bool,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallPkgResponse {
    pub id: Ulid,
}

pub async fn install_pkg(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<InstallPkgRequest>,
) -> PkgResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(), None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    let id = Ulid::new();
    tokio::task::spawn(async move {
        if let Err(err) = install_pkg_inner(
            &ctx,
            request,
            &original_uri,
            PosthogClient(posthog_client),
            raw_access_token,
        )
        .await
        {
            handle_error(&ctx, id, err.to_string()).await;
        } else {
            match WsEvent::async_finish(&ctx, id).await {
                Ok(event) => match event.publish_on_commit(&ctx).await {
                    Ok(()) => {
                        if let Err(err) = ctx.commit().await {
                            handle_error(&ctx, id, err.to_string()).await;
                        }
                    }
                    Err(err) => {
                        error!("Unable to publish ws event of finish in install pkg: {err}")
                    }
                },
                Err(err) => {
                    error!("Unable to make ws event of finish in install pkg: {err}");
                }
            }
        }

        async fn handle_error(ctx: &DalContext, id: Ulid, err: String) {
            error!("Unable to install pkg: {err}");
            match WsEvent::async_error(ctx, id, err).await {
                Ok(event) => match event.publish_on_commit(ctx).await {
                    Ok(()) => {}
                    Err(err) => error!("Unable to publish ws event of error in install pkg: {err}"),
                },
                Err(err) => {
                    error!("Unable to make ws event of error in install pkg: {err}");
                }
            }
            if let Err(err) = ctx.commit().await {
                error!("Unable to commit errors in install pkg: {err}");
            }
        }
    });

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(serde_json::to_string(&InstallPkgResponse { id })?)?)
}

async fn install_pkg_inner(
    ctx: &DalContext,
    request: InstallPkgRequest,
    original_uri: &Uri,
    PosthogClient(posthog_client): PosthogClient,
    raw_access_token: String,
) -> PkgResult<()> {
    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(PkgError::ModuleIndexNotConfigured),
    };

    let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let pkg_data = module_index_client.download_module(request.id).await?;

    let pkg = SiPkg::load_from_bytes(pkg_data)?;
    let metadata = pkg.metadata()?;
    let (_, svs, _import_skips) = import_pkg_from_pkg(
        ctx,
        &pkg,
        None, // TODO: add is_builtin option
        request.override_builtin_schema_feature_flag,
    )
    .await?;

    track(
        &posthog_client,
        ctx,
        original_uri,
        "install_pkg",
        serde_json::json!({
                    "pkg_name": metadata.name().to_owned(),
        }),
    );

    let user_pk = match ctx.history_actor() {
        HistoryActor::User(user_pk) => {
            let user = User::get_by_pk(ctx, *user_pk)
                .await?
                .ok_or(PkgError::InvalidUser(*user_pk))?;

            Some(user.pk())
        }

        HistoryActor::SystemInit => None,
    };

    match metadata.kind() {
        SiPkgKind::Module => {
            WsEvent::module_imported(ctx, svs)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
        SiPkgKind::WorkspaceBackup => {
            let workspace_pk = match metadata.workspace_pk() {
                Some(workspace_pk) => Some(WorkspacePk::from_str(workspace_pk)?),
                None => None,
            };

            WsEvent::workspace_imported(ctx, workspace_pk, user_pk)
                .await?
                .publish_on_commit(ctx)
                .await?
        }
    }

    ctx.commit().await?;
    Ok(())
}

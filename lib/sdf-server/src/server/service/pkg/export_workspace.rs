use axum::extract::OriginalUri;
use axum::http::Uri;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use dal::{DalContext, HistoryActor, User, Visibility, Workspace, WorkspacePk, WsEvent};
use telemetry::prelude::*;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};
use crate::server::tracking::track;

use super::{PkgError, PkgResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkspaceRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkspaceResponse {
    pub id: Ulid,
}

pub async fn export_workspace(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ExportWorkspaceRequest>,
) -> PkgResult<Json<ExportWorkspaceResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let id = Ulid::new();

    tokio::task::spawn(async move {
        if let Err(err) = export_workspace_inner(
            &ctx,
            &original_uri,
            PosthogClient(posthog_client),
            RawAccessToken(raw_access_token),
        )
        .await
        {
            return handle_error(&ctx, id, err.to_string()).await;
        }

        let event = match WsEvent::async_finish(&ctx, id).await {
            Ok(event) => event,
            Err(err) => {
                return error!("Unable to make ws event of finish: {err}");
            }
        };

        if let Err(err) = event.publish_on_commit(&ctx).await {
            return error!("Unable to publish ws event of finish: {err}");
        };

        if let Err(err) = ctx.commit().await {
            handle_error(&ctx, id, err.to_string()).await;
        }

        async fn handle_error(ctx: &DalContext, id: Ulid, err: String) {
            error!("Unable to export workspace: {err}");
            match WsEvent::async_error(ctx, id, err).await {
                Ok(event) => match event.publish_on_commit(ctx).await {
                    Ok(()) => {}
                    Err(err) => error!("Unable to publish ws event of error: {err}"),
                },
                Err(err) => {
                    error!("Unable to make ws event of error: {err}");
                }
            }
            if let Err(err) = ctx.commit().await {
                error!("Unable to commit errors in export workspace: {err}");
            }
        }
    });

    Ok(Json(ExportWorkspaceResponse { id }))
}

pub async fn export_workspace_inner(
    ctx: &DalContext,
    original_uri: &Uri,
    PosthogClient(posthog_client): PosthogClient,
    RawAccessToken(raw_access_token): RawAccessToken,
) -> PkgResult<()> {
    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(ctx, *user_pk).await?,
        _ => None,
    };

    let (created_by_name, created_by_email) = user
        .map(|user| (user.name().to_owned(), user.email().to_owned()))
        .unwrap_or((
            "unauthenticated user name".into(),
            "unauthenticated user email".into(),
        ));

    info!("Exporting workspace backup module");

    let workspace_pk = ctx.tenancy().workspace_pk().unwrap_or(WorkspacePk::NONE);
    let workspace = Workspace::get_by_pk(ctx, &workspace_pk)
        .await?
        .ok_or(PkgError::WorkspaceNotFound(workspace_pk))?;

    let version = Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();
    let description = "workspace backup";

    let mut exporter = dal::pkg::PkgExporter::new_workspace_exporter(
        workspace.name().as_str(),
        &created_by_email,
        &version,
        description,
    );

    let module_payload = exporter.export_as_bytes(ctx).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(PkgError::ModuleIndexNotConfigured),
    };

    let index_client =
        module_index_client::IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let response = index_client
        .upload_module(workspace.name().as_str(), &version, module_payload)
        .await?;

    track(
        &posthog_client,
        ctx,
        original_uri,
        "export_workspace",
        serde_json::json!({
                    "pkg_name": workspace.name().to_owned(),
                    "pkg_version": version,
                    "pkg_created_by_name": created_by_name,
                    "pkg_created_by_email": created_by_email,
                    "pkg_hash": response.latest_hash,
        }),
    );

    ctx.commit().await?;

    Ok(())
}

use super::{PkgError, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use chrono::Utc;
use dal::{HistoryActor, User, Visibility, Workspace, WorkspacePk, WsEvent};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkspaceRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkspaceResponse {
    pub success: bool,
    pub full_path: String,
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

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk).await?,
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
    let workspace = Workspace::get_by_pk(&ctx, &workspace_pk)
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

    let module_payload = exporter.export_as_bytes(&ctx).await?;

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
        &ctx,
        &original_uri,
        "export_workspace",
        serde_json::json!({
                    "pkg_name": workspace.name().to_owned(),
                    "pkg_version": version,
                    "pkg_created_by_name": created_by_name,
                    "pkg_created_by_email": created_by_email,
                    "pkg_hash": response.latest_hash,
        }),
    );

    // TODO: Is this really the WsEvent we want to send right now?
    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(ExportWorkspaceResponse {
        success: true,
        full_path: "Get this from module-index service".to_owned(),
    }))
}

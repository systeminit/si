use super::{PkgError, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{HistoryActor, User, Workspace};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkspaceBackupRequest {}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkspaceBackupResponse {
    pub success: bool,
}

pub async fn export_workspace_backup(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(_request): Json<ExportWorkspaceBackupRequest>,
) -> PkgResult<Json<ExportWorkspaceBackupResponse>> {
    let ctx = builder.build_head(request_ctx).await?;
    let workspace_pk = match ctx.tenancy().workspace_pk() {
        Some(pk) => pk,
        None => return Err(PkgError::InvalidWorkspaceSelection),
    };

    // TODO: seems we should already deal with this elsewhere
    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(PkgError::ModuleIndexNotConfigured),
    };

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk).await?,
        _ => return Err(PkgError::MustByLoggedIn),
    }
    .unwrap();

    // TODO: swap with call to create workspace backup

    let workspace = Workspace::get_by_pk(&ctx, &workspace_pk)
        .await?
        .ok_or(PkgError::InvalidWorkspace(workspace_pk))?;

    // which will include
    let package_payload = dal::pkg::export_pkg_as_bytes(
        &ctx,
        format!("Backup of {}", workspace.name()),
        "",        // not used for backups
        "".into(), // passing None here doesn't work ??
        user.email().to_owned(),
        [].into(),
    )
    .await?;

    dbg!("export payload created!");

    let index_client =
        module_index_client::IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let _response = index_client
        .upload_workspace_backup("backup_123", package_payload)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "export_backup",
        serde_json::json!({
                    "workspace_pk": workspace_pk
        }),
    );

    ctx.commit().await?;

    Ok(Json(ExportWorkspaceBackupResponse { success: true }))
}

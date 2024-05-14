use axum::extract::OriginalUri;
use axum::http::Uri;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use dal::{DalContext, HistoryActor, User, Visibility, Workspace, WorkspaceError, WsEvent};
use telemetry::prelude::*;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};
use crate::server::tracking::track;
use crate::service::async_route::handle_error;
use crate::service::module::{ModuleError, ModuleResult};

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
) -> ModuleResult<Json<ExportWorkspaceResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let task_id = Ulid::new();

    let workspace_pk = ctx
        .tenancy()
        .workspace_pk()
        .ok_or(ModuleError::ExportingImportingWithRootTenancy)?;
    let workspace = Workspace::get_by_pk(&ctx, &workspace_pk)
        .await?
        .ok_or(ModuleError::WorkspaceNotFound(workspace_pk))?;

    tokio::task::spawn(async move {
        if let Err(err) = export_workspace_inner(
            &ctx,
            workspace,
            &original_uri,
            PosthogClient(posthog_client),
            RawAccessToken(raw_access_token),
        )
        .await
        {
            return handle_error(&ctx, original_uri, task_id, err).await;
        }

        let event = match WsEvent::async_finish(&ctx, task_id).await {
            Ok(event) => event,
            Err(err) => {
                return handle_error(&ctx, original_uri, task_id, err).await;
            }
        };

        if let Err(err) = event.publish_immediately(&ctx).await {
            handle_error(&ctx, original_uri, task_id, err).await;
        };
    });

    Ok(Json(ExportWorkspaceResponse { id: task_id }))
}

pub async fn export_workspace_inner(
    ctx: &DalContext,
    workspace: Workspace,
    original_uri: &Uri,
    PosthogClient(posthog_client): PosthogClient,
    RawAccessToken(raw_access_token): RawAccessToken,
) -> ModuleResult<()> {
    info!("Exporting workspace backup");
    let version = Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();

    let index_client = {
        let module_index_url = match ctx.module_index_url() {
            Some(url) => url,
            None => return Err(ModuleError::ModuleIndexNotConfigured),
        };

        module_index_client::IndexClient::new(module_index_url.try_into()?, &raw_access_token)
    };

    let workspace_payload = workspace.generate_export_data(ctx, &version).await?;

    index_client
        .upload_workspace(workspace.name().as_str(), &version, workspace_payload)
        .await?;

    // Track
    {
        let created_by = if let HistoryActor::User(user_pk) = ctx.history_actor() {
            let user = User::get_by_pk(ctx, *user_pk)
                .await?
                .ok_or(WorkspaceError::InvalidUser(*user_pk))?;

            user.email().clone()
        } else {
            "SystemInit".to_string()
        };

        track(
            &posthog_client,
            ctx,
            original_uri,
            "export_workspace",
            serde_json::json!({
                "pkg_name": workspace.name().to_owned(),
                "pkg_version": version,
                "pkg_created_by_email": created_by,
            }),
        );
    }

    Ok(())
}

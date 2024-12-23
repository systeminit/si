use axum::{
    extract::{Host, OriginalUri, Path},
    http::Uri,
    Json,
};
use chrono::Utc;
use dal::{DalContext, HistoryActor, User, Workspace, WorkspacePk, WsEvent};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;
use telemetry::prelude::info;
use ulid::Ulid;

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken},
    service::async_route::handle_error,
    track,
};

use super::{WorkspaceAPIError, WorkspaceAPIResult};

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
    Host(host_name): Host,
    Path(_workspace_pk): Path<WorkspacePk>,
) -> WorkspaceAPIResult<Json<ExportWorkspaceResponse>> {
    let ctx = builder.build_head(request_ctx).await?;

    let current_workspace = {
        let workspace_pk = ctx
            .tenancy()
            .workspace_pk_opt()
            .ok_or(WorkspaceAPIError::ExportingImportingWithRootTenancy)?;
        Workspace::get_by_pk(&ctx, &workspace_pk)
            .await?
            .ok_or(WorkspaceAPIError::WorkspaceNotFound(workspace_pk))?
    };

    let task_id = Ulid::new();

    tokio::task::spawn(async move {
        if let Err(err) = export_workspace_inner(
            &ctx,
            current_workspace,
            &original_uri,
            &host_name,
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
    current_workspace: Workspace,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): PosthogClient,
    RawAccessToken(raw_access_token): RawAccessToken,
) -> WorkspaceAPIResult<()> {
    info!("Exporting workspace backup");
    let version = Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();

    let index_client = {
        let module_index_url = match ctx.module_index_url() {
            Some(url) => url,
            None => return Err(WorkspaceAPIError::ModuleIndexNotConfigured),
        };

        module_index_client::ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token)
    };

    let workspace_payload = current_workspace
        .generate_export_data(ctx, &version)
        .await?;

    index_client
        .upload_workspace(
            current_workspace.name().as_str(),
            &version,
            workspace_payload,
        )
        .await?;

    let workspace_id = *current_workspace.pk();
    ctx.write_audit_log(
        AuditLogKind::ExportWorkspace {
            id: workspace_id,
            name: current_workspace.name().clone(),
            version: version.clone(),
        },
        current_workspace.name().to_string(),
    )
    .await?;

    // Track
    {
        let created_by = if let HistoryActor::User(user_pk) = ctx.history_actor() {
            let user = User::get_by_pk(ctx, *user_pk)
                .await?
                .ok_or(WorkspaceAPIError::InvalidUser(*user_pk))?;

            user.email().clone()
        } else {
            "SystemInit".to_string()
        };

        track(
            &posthog_client,
            ctx,
            original_uri,
            host_name,
            "export_workspace",
            serde_json::json!({
                "pkg_name": current_workspace.name().to_owned(),
                "pkg_version": version,
                "pkg_created_by_email": created_by,
            }),
        );
    }

    Ok(())
}

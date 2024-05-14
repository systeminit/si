use axum::extract::OriginalUri;
use axum::http::Uri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use dal::{DalContext, Workspace, WsEvent};
use module_index_client::IndexClient;
use telemetry::prelude::info;

use crate::server::extract::RawAccessToken;
use crate::service::async_route::handle_error;
use crate::{
    server::extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::module::ModuleError,
};

use super::ModuleResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallWorkspaceRequest {
    pub id: Ulid,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallWorkspaceResponse {
    pub id: Ulid,
}

pub async fn install_workspace(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<InstallWorkspaceRequest>,
) -> ModuleResult<impl IntoResponse> {
    let ctx = builder.build_head(request_ctx).await?;

    let workspace = {
        let workspace_pk = ctx
            .tenancy()
            .workspace_pk()
            .ok_or(ModuleError::ExportingImportingWithRootTenancy)?;
        Workspace::get_by_pk(&ctx, &workspace_pk)
            .await?
            .ok_or(ModuleError::WorkspaceNotFound(workspace_pk))?
    };

    let id = Ulid::new();
    tokio::task::spawn(async move {
        if let Err(err) = install_workspace_inner(
            &ctx,
            request,
            workspace,
            &original_uri,
            PosthogClient(posthog_client),
            raw_access_token,
        )
        .await
        {
            handle_error(&ctx, original_uri, id, err).await;
        } else {
            match WsEvent::async_finish_workspace(&ctx, id).await {
                Ok(event) => {
                    if let Err(err) = event.publish_immediately(&ctx).await {
                        handle_error(&ctx, original_uri, id, err).await;
                    }
                }
                Err(err) => {
                    handle_error(&ctx, original_uri, id, err).await;
                }
            }
        }
    });

    Ok(Json(InstallWorkspaceResponse { id }))
}

async fn install_workspace_inner(
    ctx: &DalContext,
    request: InstallWorkspaceRequest,
    mut workspace: Workspace,
    _original_uri: &Uri,
    PosthogClient(_posthog_client): PosthogClient,
    raw_access_token: String,
) -> ModuleResult<()> {
    info!("Importing workspace backup");
    let workspace_data = {
        let module_index_url = match ctx.module_index_url() {
            Some(url) => url,
            None => return Err(ModuleError::ModuleIndexNotConfigured),
        };
        let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
        module_index_client.download_workspace(request.id).await?
    };

    workspace.import(ctx, workspace_data).await?;

    ctx.commit_no_rebase().await?;

    Ok(())
}

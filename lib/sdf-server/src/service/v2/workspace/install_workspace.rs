use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
    http::Uri,
};
use dal::{
    DalContext,
    Workspace,
    WorkspacePk,
    WsEvent,
};
use module_index_client::ModuleIndexClient;
use sdf_core::async_route::handle_error;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use si_pkg::WorkspaceExportContentV0;
use telemetry::prelude::info;
use ulid::Ulid;

use super::{
    WorkspaceAPIError,
    WorkspaceAPIResult,
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
pub struct InstallWorkspaceResponse {
    pub id: Ulid,
}

pub async fn install_workspace(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path(req_workspace_pk): Path<WorkspacePk>,
) -> WorkspaceAPIResult<Json<InstallWorkspaceResponse>> {
    let ctx = builder.build_head(request_ctx).await?;

    let current_workspace = {
        let workspace_pk = ctx
            .tenancy()
            .workspace_pk_opt()
            .ok_or(WorkspaceAPIError::RootTenancyInstallAttempt)?;
        Workspace::get_by_pk(&ctx, workspace_pk).await?
    };

    let id = Ulid::new();

    tokio::task::spawn(async move {
        match install_workspace_inner(
            &ctx,
            req_workspace_pk,
            current_workspace,
            &original_uri,
            &host_name,
            PosthogClient(posthog_client),
            raw_access_token,
        )
        .await
        {
            Err(err) => {
                handle_error(&ctx, original_uri, id, err).await;
            }
            _ => match WsEvent::async_finish_workspace(&ctx, id).await {
                Ok(event) => {
                    if let Err(err) = event.publish_immediately(&ctx).await {
                        handle_error(&ctx, original_uri, id, err).await;
                    }
                }
                Err(err) => {
                    handle_error(&ctx, original_uri, id, err).await;
                }
            },
        }
    });

    Ok(Json(InstallWorkspaceResponse { id }))
}

async fn install_workspace_inner(
    ctx: &DalContext,
    workspace_pk: WorkspacePk,
    mut current_workspace: Workspace,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): PosthogClient,
    raw_access_token: String,
) -> WorkspaceAPIResult<()> {
    info!("Importing workspace backup");
    let workspace_data = {
        let module_index_url = match ctx.module_index_url() {
            Some(url) => url,
            None => return Err(WorkspaceAPIError::ModuleIndexUrlNotSet),
        };
        let module_index_client =
            ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token)?;
        module_index_client
            .download_workspace(workspace_pk.into())
            .await?
    };

    current_workspace
        .import(ctx, workspace_data.clone())
        .await?;

    let WorkspaceExportContentV0 {
        change_sets: _,
        content_store_values: _,
        metadata,
    } = workspace_data.into_latest();
    let workspace_id = *current_workspace.pk();

    ctx.write_audit_log(
        AuditLogKind::InstallWorkspace {
            id: workspace_id,
            name: current_workspace.name().clone(),
            version: metadata.version.clone(),
        },
        current_workspace.name().to_string(),
    )
    .await?;

    // Track
    {
        track(
            &posthog_client,
            ctx,
            original_uri,
            host_name,
            "import_workspace",
            serde_json::json!({
                "pkg_name": current_workspace.name().to_owned(),
                "pkg_version": metadata.version.clone(),
                "pkg_created_by_email": metadata.created_by,
                "pkg_created_at":  metadata.created_at,
            }),
        );
    }

    ctx.commit_no_rebase().await?;

    Ok(())
}

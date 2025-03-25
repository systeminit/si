use crate::extract::{HandlerContext, PosthogClient};
use crate::service::v2::AccessBuilder;
use crate::track;

use axum::extract::{Host, OriginalUri, Path, State};
use axum::Json;
use dal::workspace_integrations::{WorkspaceIntegration, WorkspaceIntegrationId};
use dal::{HistoryActor, UserPk, WorkspacePk};
use permissions::{Permission, PermissionBuilder};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;

use super::{AppState, IntegrationsError, IntegrationsResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIntegrationRequest {
    slack_webhook_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIntegrationResponse {
    pub integration: WorkspaceIntegration,
}

#[allow(clippy::too_many_arguments)]
pub async fn update_integration(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    State(mut state): State<AppState>,
    Path((workspace_pk, workspace_integration_id)): Path<(WorkspacePk, WorkspaceIntegrationId)>,
    Json(request): Json<UpdateIntegrationRequest>,
) -> IntegrationsResult<Json<UpdateIntegrationResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let spicedb_client = state
        .spicedb_client()
        .ok_or(IntegrationsError::SpiceDbClientNotFound)?;

    let user_pk: UserPk = match ctx.history_actor() {
        HistoryActor::User(user_id) => *user_id,
        _ => return Err(IntegrationsError::InvalidUser),
    };
    let has_permission = PermissionBuilder::new()
        .workspace_object(workspace_pk)
        .permission(Permission::Approve)
        .user_subject(user_pk)
        .has_permission(spicedb_client)
        .await?;
    if !has_permission {
        return Err(IntegrationsError::UserUnableToApproveIntegration(user_pk));
    }

    let mut integration = WorkspaceIntegration::get_by_pk(&ctx, workspace_integration_id)
        .await?
        .ok_or(IntegrationsError::IntegrationNotFound(
            workspace_integration_id,
        ))?;

    if let Some(webhook_url) = request.slack_webhook_url {
        let old_url = integration.slack_webhook_url().unwrap_or_default();
        integration
            .update_webhook_url(&ctx, webhook_url.clone())
            .await?;

        // We don't want to track the webhook URL change
        // we only want to track that the feature was interacted with
        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "update_workspace_integration",
            serde_json::json!({}),
        );

        ctx.write_audit_log_to_head(
            AuditLogKind::WorkspaceIntegration {
                old_slack_webhook_url: old_url,
                new_slack_webhook_url: webhook_url.clone(),
            },
            "slack_webhook_url".to_string(),
        )
        .await?;
    }

    ctx.commit_no_rebase().await?;

    Ok(Json(UpdateIntegrationResponse { integration }))
}

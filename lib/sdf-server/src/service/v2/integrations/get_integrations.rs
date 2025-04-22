use crate::extract::{HandlerContext, PosthogClient};
use crate::service::v2::AccessBuilder;
use axum::Json;
use axum::extract::{Host, OriginalUri};
use dal::workspace_integrations::WorkspaceIntegration;
use serde::{Deserialize, Serialize};

use super::IntegrationsResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetIntegrationResponse {
    pub integration: Option<WorkspaceIntegration>,
}

pub async fn get_integration(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
) -> IntegrationsResult<Json<GetIntegrationResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let integration = WorkspaceIntegration::get_integrations_for_workspace_pk(&ctx).await?;

    Ok(Json(GetIntegrationResponse { integration }))
}

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::workspace_integrations::WorkspaceIntegration;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    IntegrationResponse,
    IntegrationsResult,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::v2::AccessBuilder,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceIntegrationResponse {
    pub integration: Option<IntegrationResponse>,
}

pub async fn get_integration(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
) -> IntegrationsResult<Json<WorkspaceIntegrationResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let integration = WorkspaceIntegration::get_integrations_for_workspace_pk(&ctx)
        .await?
        .map(|i| i.into());

    Ok(Json(WorkspaceIntegrationResponse { integration }))
}

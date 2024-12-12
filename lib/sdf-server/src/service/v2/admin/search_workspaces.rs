use anyhow::Result;
use axum::{extract::Query, Json};
use dal::Workspace;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::service::v2::admin::{AdminUserContext, AdminWorkspace};

const SEARCH_LIMIT: usize = 50;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SearchWorkspacesRequest {
    query: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SearchWorkspacesResponse {
    workspaces: Vec<AdminWorkspace>,
}

#[instrument(name = "admin.search_workspaces", skip_all)]
pub async fn search_workspaces(
    AdminUserContext(ctx): AdminUserContext,
    Query(request): Query<SearchWorkspacesRequest>,
) -> Result<Json<SearchWorkspacesResponse>> {
    let workspaces = Workspace::search(&ctx, request.query.as_deref(), SEARCH_LIMIT)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(SearchWorkspacesResponse { workspaces }))
}

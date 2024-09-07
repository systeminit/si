use axum::{extract::Query, response::IntoResponse, Json};
use telemetry::prelude::*;

use dal::Workspace;
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{AdminAPIResult, AdminWorkspace};

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
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Query(request): Query<SearchWorkspacesRequest>,
) -> AdminAPIResult<impl IntoResponse> {
    let ctx = builder.build_head(access_builder).await?;

    let workspaces = Workspace::search(&ctx, request.query.as_deref(), SEARCH_LIMIT)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(SearchWorkspacesResponse { workspaces }))
}

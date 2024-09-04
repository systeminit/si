use std::collections::HashMap;

use axum::{response::IntoResponse, Json};

use dal::{Workspace, WorkspacePk};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{AdminAPIResult, AdminWorkspace};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ListWorkspacesResponse {
    workspaces: HashMap<WorkspacePk, AdminWorkspace>,
}

pub async fn list_workspaces(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
) -> AdminAPIResult<impl IntoResponse> {
    let ctx = builder.build_head(access_builder).await?;

    let workspaces = Workspace::list_all(&ctx)
        .await?
        .into_iter()
        .map(|workspace| (*workspace.pk(), workspace.into()))
        .collect();

    Ok(Json(ListWorkspacesResponse { workspaces }))
}

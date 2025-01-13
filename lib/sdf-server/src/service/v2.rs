use axum::{
    async_trait,
    extract::{FromRequestParts, Path},
    http::{request::Parts, Request},
    middleware::Next,
    response::Response,
    Router,
};
use dal::WorkspacePk;
use serde::Deserialize;

use crate::{
    extract::{
        bad_request,
        workspace::{TargetWorkspaceId, TargetWorkspaceIdFromHeader, WorkspaceAuthorization},
        ErrorResponse,
    },
    AppState,
};

pub mod admin;
pub mod audit_log;
pub mod change_set;
pub mod func;
pub mod integrations;
pub mod management;
pub mod module;
pub mod variant;
pub mod view;
pub mod workspace;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/admin", admin::v2_routes(state.clone()))
        .nest("/workspaces/:workspace_id", workspace_routes(state))
}

fn workspace_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/", workspace::v2_routes())
        .nest("/change-sets", change_set::change_sets_routes())
        .nest(
            "/change-sets/:change_set_id",
            change_set::change_set_routes(state.clone())
                .nest("/audit-logs", audit_log::v2_routes())
                .nest("/funcs", func::v2_routes())
                .nest("/modules", module::v2_routes())
                .nest("/schema-variants", variant::v2_routes())
                .nest("/management", management::v2_routes())
                .nest("/views", view::v2_routes()),
        )
        .nest("/integrations", integrations::v2_routes())
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            target_workspace_id_from_path,
        ))
}

/// An authorized user + workspace
#[derive(Clone, Debug, derive_more::Deref, derive_more::Into)]
pub struct AccessBuilder(pub dal::AccessBuilder);

#[async_trait]
impl FromRequestParts<AppState> for AccessBuilder {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Ensure the endpoint is authorized
        let auth = WorkspaceAuthorization::from_request_parts(parts, state).await?;

        Ok(Self(auth.into()))
    }
}

// Stash the TargetWorkspaceId so that authorization has it
async fn target_workspace_id_from_path<B>(
    Path(WorkspaceIdFromPath { workspace_id }): Path<WorkspaceIdFromPath>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, ErrorResponse> {
    // Check against header if it exists
    if TargetWorkspaceIdFromHeader::extract(request.headers())?
        .is_some_and(|header_workspace_id| header_workspace_id != workspace_id)
    {
        return Err(bad_request("Workspace ID in path does not match header"));
    }

    request
        .extensions_mut()
        .insert(TargetWorkspaceId(workspace_id));
    Ok(next.run(request).await)
}

#[derive(Deserialize)]
struct WorkspaceIdFromPath {
    workspace_id: WorkspacePk,
}

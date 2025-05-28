use axum::{
    RequestPartsExt,
    Router,
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    middleware,
};

use crate::{
    AppState,
    extract::{
        ErrorResponse,
        change_set::TargetChangeSetIdFromPath,
        workspace::{
            TargetWorkspaceIdFromPath,
            WorkspaceAuthorization,
        },
    },
};

pub mod action;
pub mod admin;
pub mod approval_requirement_definition;
pub mod audit_log;
pub mod change_set;
pub mod component;
pub mod fs;
pub mod func;
pub mod index;
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
                .nest("/components", component::v2_routes())
                .nest("/funcs", func::v2_routes())
                .nest("/modules", module::v2_routes())
                .nest("/schema-variants", variant::v2_routes())
                .nest("/management", management::v2_routes())
                .nest("/views", view::v2_routes())
                .nest("/action", action::v2_routes())
                .nest(
                    "/approval-requirement-definitions",
                    approval_requirement_definition::v2_routes(),
                )
                .route_layer(middleware::from_extractor::<TargetChangeSetIdFromPath>()),
        )
        .nest("/fs", fs::fs_routes(state.clone()))
        .nest("/integrations", integrations::v2_routes())
        .route_layer(middleware::from_extractor::<TargetWorkspaceIdFromPath>())
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
        let WorkspaceAuthorization { ctx, .. } = parts.extract_with_state(state).await?;
        Ok(Self(ctx.access_builder()))
    }
}

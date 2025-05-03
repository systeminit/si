use axum::{
    Router,
    http::StatusCode,
    middleware,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        delete,
        get,
        post,
    },
};
use thiserror::Error;

use super::common::ErrorIntoResponse;
use crate::{
    AppState,
    extract::{
        change_set::TargetChangeSetIdFromPath,
        workspace::{
            AuthorizedForAutomationRole,
            TargetWorkspaceIdFromPath,
        },
    },
    middleware::WorkspacePermissionLayer,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("workspace error: {0}")]
    Workspace(#[from] dal::WorkspaceError),
}

impl ErrorIntoResponse for WorkspaceError {
    fn status_and_message(&self) -> (StatusCode, String) {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
    }
}

impl IntoResponse for WorkspaceError {
    fn into_response(self) -> Response {
        self.to_api_response()
    }
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new().nest(
        "/:workspace_id",
        Router::new()
            .nest(
                "/change-sets",
                Router::new()
                    .route("/", post(super::change_sets::create::create_change_set))
                    .route("/", get(super::change_sets::list::list_change_sets))
                    .nest(
                        "/:change_set_id",
                        Router::new()
                            .route("/", get(super::change_sets::get::get_change_set))
                            .route("/", delete(super::change_sets::delete::abandon_change_set))
                            .nest("/components", super::components::routes())
                            .nest("/schema", super::schema::routes())
                            .nest("/funcs", super::funcs::routes())
                            .nest("/actions", super::actions::routes())
                            .nest("/secrets", super::secrets::routes())
                            .route(
                                "/request_approval",
                                post(super::change_sets::request_approval::request_approval),
                            )
                            .route(
                                "/force_apply",
                                post(super::change_sets::force_apply::force_apply).route_layer(
                                    WorkspacePermissionLayer::new(
                                        state.clone(),
                                        permissions::Permission::Approve,
                                    ),
                                ),
                            )
                            .route(
                                "/merge_status",
                                get(super::change_sets::merge_status::merge_status),
                            )
                            .route_layer(middleware::from_extractor::<TargetChangeSetIdFromPath>()),
                    ),
            )
            .route_layer(middleware::from_extractor_with_state::<
                AuthorizedForAutomationRole,
                AppState,
            >(state))
            .route_layer(middleware::from_extractor::<TargetWorkspaceIdFromPath>()),
    )
}

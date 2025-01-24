use axum::{middleware, Router};

use crate::{
    extract::workspace::{AuthorizedForAutomationRole, TargetWorkspaceIdFromPath},
    AppState,
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new().nest(
        "/:workspace_id",
        Router::new()
            .nest("/change-sets", super::change_sets::routes(state.clone()))
            .route_layer(middleware::from_extractor_with_state::<
                AuthorizedForAutomationRole,
                AppState,
            >(state))
            .route_layer(middleware::from_extractor::<TargetWorkspaceIdFromPath>()),
    )
}

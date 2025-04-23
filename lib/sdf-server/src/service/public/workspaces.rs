use axum::{
    Router,
    middleware,
};

use crate::{
    AppState,
    extract::workspace::{
        AuthorizedForAutomationRole,
        TargetWorkspaceIdFromPath,
    },
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

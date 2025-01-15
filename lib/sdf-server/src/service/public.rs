use axum::Router;

use crate::AppState;

mod change_sets;
mod components;
mod management;
mod workspaces;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new().nest(
        "/v0",
        Router::new().nest("/workspaces", workspaces::routes(state)),
    )
}

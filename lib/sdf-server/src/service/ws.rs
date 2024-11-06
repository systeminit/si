use axum::{http::StatusCode, response::IntoResponse, response::Response, routing::get, Router};

use axum_util::app_state::AppState;

pub mod crdt;
pub mod workspace_updates;

pub use axum_util::service::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/workspace_updates",
            get(workspace_updates::workspace_updates),
        )
        .route("/crdt", get(crdt::crdt))
}

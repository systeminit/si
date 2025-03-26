use axum::{routing::get, Router};

use crate::AppState;

pub mod bifrost;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/bifrost", get(bifrost::bifrost_handler))
        .with_state(state)
}

use axum::Router;

use crate::AppState;

pub mod summary;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/summary", axum::routing::get(summary::graph_summary))
}
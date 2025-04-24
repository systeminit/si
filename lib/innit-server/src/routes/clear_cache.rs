use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use innit_core::RefreshCacheResponse;
use telemetry::tracing::info;

use crate::app_state::AppState;

pub async fn clear_cache_route(
    State(AppState {
        parameter_cache, ..
    }): State<AppState>,
) -> Result<Json<RefreshCacheResponse>, StatusCode> {
    parameter_cache.clear_cache().await;

    info!("Cleared parameter cache.");
    Ok(Json(RefreshCacheResponse { success: true }))
}

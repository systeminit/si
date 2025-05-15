use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use bedrock_core::TestProfileResponse;
use telemetry::tracing::info;

use crate::{
    app_state::AppState,
    profiles::load_profiles,
};

/// Lists all available test profiles.
pub async fn profiles_route(
    State(_): State<AppState>,
) -> Result<Json<TestProfileResponse>, StatusCode> {
    info!("Received request to list test profiles");

    let profiles = load_profiles();
    Ok(Json(profiles))
}

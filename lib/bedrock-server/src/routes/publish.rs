// src/routes/publish.rs
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use bedrock_core::{ArtifactStoreConfig, PublishRequest, PublishResult};
use serde_json::json;
use telemetry::tracing::info;

use crate::{app_state::AppState, artifacts::publish_artifact};

/// Axum-compatible route handler
pub async fn publish_route(
    State(_state): State<AppState>, // Include if you need shared state
    Json(payload): Json<PublishRequest>,
) -> Result<Json<PublishResult>, (StatusCode, Json<PublishResult>)> {
    info!("Received artifact publish request: {}", payload.artifact_id);

    let config = ArtifactStoreConfig {
        variant: "s3".to_string(),
        metadata: json!({
            "bucketName": "artifacts.systeminit.com"
        }),
    };

    let result = publish_artifact(
        &payload.artifact_id,
        payload.metadata.clone(),
        &config,
    )
    .await;

    if result.success {
        Ok(Json(result))
    } else {
        let status = classify_failure(&result);
        Err((status, Json(result)))
    }
}

fn classify_failure(result: &PublishResult) -> StatusCode {
    let msg = result.message.to_lowercase();
    if msg.contains("timeout") {
        StatusCode::FAILED_DEPENDENCY // i.e. the service is fine, but the upload was not
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

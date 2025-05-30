// src/routes/prepare.rs
use axum::{
    extract::{
        Json,
        State,
    },
    http::StatusCode,
};
use bedrock_core::{
    PrepareRequest,
    PrepareResult,
};
use telemetry::tracing::info;

use crate::{
    app_state::AppState,
    artifacts::{
        clear_nats,
        prepare_databases,
        resolve_test,
    },
};

pub async fn prepare_route(
    State(state): State<AppState>,
    Json(payload): Json<PrepareRequest>,
) -> Result<Json<PrepareResult>, (StatusCode, Json<PrepareResult>)> {
    let recording_id = payload.recording_id;
    info!("Received test prepare request: {}", recording_id);

    let artifact_config = state.artifact_config;
    let start_time = std::time::Instant::now();

    let database_sql_dumps = match resolve_test(&recording_id, artifact_config).await {
        Ok(paths) => paths,
        Err(_e) => {
            let result = PrepareResult {
                success: false,
                message: format!("Failed to find/resolve recording_id: {}", &recording_id),
                recording_id,
                duration_ms: Some(start_time.elapsed().as_millis() as u64),
                output: None,
            };
            return Err((StatusCode::NOT_FOUND, Json(result)));
        }
    };

    if let Err(e) = prepare_databases(database_sql_dumps).await {
        let result = PrepareResult {
            success: false,
            message: format!("Failed to prepare requested databases: {}", e),
            recording_id,
            duration_ms: Some(start_time.elapsed().as_millis() as u64),
            output: None,
        };
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(result)));
    }

    if let Err(e) = clear_nats(&state.nats).await {
        let result = PrepareResult {
            success: false,
            message: format!("Failed to clear nats: {}", e),
            recording_id,
            duration_ms: Some(start_time.elapsed().as_millis() as u64),
            output: None,
        };
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(result)));
    }

    if let Some(ref meta) = payload.metadata {
        info!("messages: {} | timeout: {}", meta.messages, meta.timeout);
    }

    let duration = start_time.elapsed().as_millis() as u64;

    let response = PrepareResult {
        success: true,
        message: format!(
            "Preparation complete for {}, please conduct the test and hit /tests to execute test",
            &recording_id
        ),
        recording_id,
        duration_ms: Some(duration),
        output: None,
    };

    Ok(Json(response))
}

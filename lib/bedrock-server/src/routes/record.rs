// src/routes/record.rs
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use bedrock_core::{RecordRequest, RecordResult};
use telemetry::tracing::info;
use uuid::Uuid;
use crate::{
    app_state::AppState, 
    artifacts::dump_databases,
    artifacts::configure_nats,
    artifacts::capture_nats, 
};

pub async fn start_recording_route(
    State(state): State<AppState>,
    Json(payload): Json<RecordRequest>,
) -> Result<Json<RecordResult>, (StatusCode, Json<RecordResult>)> {
    
    // If a recording ID wasn't passed, generate one
    let recording_id = payload
        .recording_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    info!("Received artifact record request:");
    info!("Recording ID: {}", recording_id);

    let start_time = std::time::Instant::now();

    if let Some(ref nats_streams) = payload.nats {
        info!("NATS Streams: {:?}", nats_streams);
        if let Err(e) = configure_nats(&state.nats, nats_streams, &recording_id).await {
            let result = RecordResult {
                success: false,
                message: format!("Failed to setup nats: {}", e),
                recording_id,
                duration_ms: Some(start_time.elapsed().as_millis() as u64),
                output: None,
            };
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(result)));
        }
    }

    if let Some(ref postgres_dbs) = payload.postgres {
        info!("Postgres DBs: {:?}", postgres_dbs);
        if let Err(e) = dump_databases(postgres_dbs, &recording_id, "start").await {
            let result = RecordResult {
                success: false,
                message: format!("Failed to dump requested databases: {}", e),
                recording_id,
                duration_ms: Some(start_time.elapsed().as_millis() as u64),
                output: None,
            };
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(result)));
        }
    }

    if let Some(ref meta) = payload.metadata {
        info!("messages: {} | timeout: {}", meta.messages, meta.timeout);
    }

    let duration = start_time.elapsed().as_millis() as u64;

    let response = RecordResult {
        success: true,
        message: format!("Recording started for {}, please conduct the test and hit /stop to finalise capture", &recording_id),
        recording_id,
        duration_ms: Some(duration),
        output: None,
    };

    Ok(Json(response))
}

pub async fn stop_recording_route(
    State(state): State<AppState>,
    Json(payload): Json<RecordRequest>,
) -> Result<Json<RecordResult>, (StatusCode, Json<RecordResult>)> {
    
    // If a recording ID wasn't passed, generate one
    let recording_id = payload
        .recording_id
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    info!("Received artifact stop recording request:");
    info!("Recording ID: {}", recording_id);

    let start_time = std::time::Instant::now();

    if let Some(ref nats_streams) = payload.nats {
        info!("NATS Streams: {:?}", nats_streams);
        if let Err(e) = capture_nats(&state.nats, nats_streams, &recording_id).await {
            let result = RecordResult {
                success: false,
                message: format!("Failed to capture messages on nats: {}", e),
                recording_id,
                duration_ms: Some(start_time.elapsed().as_millis() as u64),
                output: None,
            };
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(result)));
        }
    }

    if let Some(ref postgres_dbs) = payload.postgres {
        info!("Postgres DBs: {:?}", postgres_dbs);
        if let Err(e) = dump_databases(postgres_dbs, &recording_id, "end").await {
            let result = RecordResult {
                success: false,
                message: format!("Failed to dump requested databases: {}", e),
                recording_id,
                duration_ms: Some(start_time.elapsed().as_millis() as u64),
                output: None,
            };
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(result)));
        }
    }

    if let Some(ref meta) = payload.metadata {
        info!("messages: {} | timeout: {}", meta.messages, meta.timeout);
    }

    let duration = start_time.elapsed().as_millis() as u64;

    let response = RecordResult {
        success: true,
        message: format!("Recording stopped, please see output directory for content for recording_id {}", recording_id),
        recording_id,
        duration_ms: Some(duration),
        output: None,
    };

    Ok(Json(response))
}

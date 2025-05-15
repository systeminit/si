use axum::{
    extract::{
        Json,
        State,
    },
    http::StatusCode,
};
use bedrock_core::{
    ExecutionParameters,
    Parameters,
    TestResult,
};
use serde::Deserialize;
use telemetry::tracing::info;

use crate::{
    app_state::AppState,
    profiles::run_test,
};

/// Payload expected from the client to trigger a test run.
#[derive(Debug, Deserialize)]
pub struct RunTestRequest {
    pub recording_id: String,
    pub parameters: Parameters,
    pub execution_parameters: ExecutionParameters,
}

/// HTTP handler to execute a registered test profile.
pub async fn execute_tests_route(
    State(app_state): State<AppState>,
    Json(payload): Json<RunTestRequest>,
) -> Result<Json<TestResult>, (StatusCode, Json<TestResult>)> {
    info!(
        "Received test execution request; recording_id={}",
        payload.recording_id
    );

    match run_test(
        &payload.recording_id,
        &payload.parameters,
        &payload.execution_parameters,
        &app_state.nats,
    )
    .await
    {
        Some(result) => {
            if result.success {
                Ok(Json(result))
            } else {
                let status = classify_failure(&result);
                Err((status, Json(result)))
            }
        }
        None => {
            info!(
                "Test not found: recording_id={}",
                payload.recording_id
            );
            Err((StatusCode::NOT_FOUND, Json(TestResult {
                success: false,
                message: "Test not found".into(),
                duration_ms: None,
                output: None,
            })))
        }
    }
}

fn classify_failure(result: &TestResult) -> StatusCode {
    let msg = result.message.to_lowercase();
    if msg.contains("missing rebase batch") || msg.contains("timeout") || msg.contains("invalid") || msg.contains("not found") {
        StatusCode::FAILED_DEPENDENCY // i.e. the service is fine, but the one getting tested is not
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

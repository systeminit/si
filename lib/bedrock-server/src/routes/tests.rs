use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use telemetry::tracing::info;
use serde::Deserialize;

use bedrock_core::{TestResult, Parameters, ExecutionParameters};
use crate::{
    app_state::AppState,
    profiles::run_test, // dispatcher function you'll define in `profiles/mod.rs`
};

/// Payload expected from the client to trigger a test run.
#[derive(Debug, Deserialize)]
pub struct RunTestRequest {
    pub instance_id: String,
    pub service: String,
    pub test: String,
    pub parameters: Parameters,
    
    #[serde(rename = "executionParameters")]
    pub execution_parameters: ExecutionParameters,
}

/// HTTP handler to execute a registered test profile.
pub async fn tests_route(
    State(appState): State<AppState>,
    Json(payload): Json<RunTestRequest>,
) -> Result<Json<TestResult>, StatusCode> {
    info!(
        "Received test execution request; instance_id={}, service={}, test={}",
        payload.instance_id, payload.service, payload.test
    );

    match run_test(
        &payload.service,
        &payload.test,
        &payload.parameters,
        &payload.execution_parameters,
        &appState.nats,
    ).await  {
        Some(result) => Ok(Json(result)),
        None => {
            info!(
                "Test not found: service={}, test={}",
                payload.service, payload.test
            );
            Err(StatusCode::NOT_FOUND)
        }
    }
}

// src/routes/publish.rs
use std::time::{
    Duration,
    SystemTime,
};

use aws_credential_types::Credentials as AwsTypeCredentials;
use axum::{
    extract::{
        Json,
        State,
    },
    http::StatusCode,
};
use bedrock_core::{
    ArtifactStoreConfig,
    PublishRequest,
    PublishResult,
};
use s3::creds::Credentials;
use serde_json::json;
use telemetry::tracing::info;

use crate::{
    app_state::AppState,
    artifacts::publish_artifact,
};

pub fn convert_s3_creds_to_aws(creds: &Credentials) -> Option<AwsTypeCredentials> {
    let access_key = creds.access_key.as_ref()?.trim();
    let secret_key = creds.secret_key.as_ref()?.trim();

    if access_key.is_empty() || secret_key.is_empty() {
        return None;
    }

    let session_token = creds
        .security_token
        .as_ref()
        .or(creds.session_token.as_ref())
        .map(|s| s.to_owned());

    let expires_after = creds
        .expiration
        .map(|dt| SystemTime::UNIX_EPOCH + Duration::from_secs(dt.unix_timestamp().max(0) as u64));

    Some(AwsTypeCredentials::new(
        access_key.to_string(),
        secret_key.to_string(),
        session_token,
        expires_after,
        "converted-from-s3-creds",
    ))
}

pub async fn publish_route(
    State(state): State<AppState>,
    Json(payload): Json<PublishRequest>,
) -> Result<Json<PublishResult>, (StatusCode, Json<PublishResult>)> {
    info!(
        "Received artifact publish request: {}",
        payload.recording_id
    );

    let aws_credentials = match convert_s3_creds_to_aws(&state.aws_credentials) {
        Some(c) => c,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(PublishResult {
                    success: false,
                    message: "Invalid or missing AWS credentials".into(),
                    duration_ms: None,
                    output: None,
                }),
            ));
        }
    };

    let config = ArtifactStoreConfig {
        variant: "s3".to_string(),
        metadata: json!({
            "bucketName": "si-artifacts-prod"
        }),
    };

    let result = publish_artifact(&payload.recording_id, aws_credentials, &config).await;

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
        StatusCode::FAILED_DEPENDENCY // i.e. but the upload was not fine
    } else if msg.contains("already exists") {
        StatusCode::CONFLICT // i.e. the upload already exists
    } else if msg.contains("does not exist") {
        StatusCode::NOT_FOUND // i.e. the artifact doesn't exist locally
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

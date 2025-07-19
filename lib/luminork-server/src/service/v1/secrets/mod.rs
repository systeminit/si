use std::collections::HashMap;

use axum::{
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::{
        delete,
        get,
        post,
        put,
    },
};
use dal::{
    PublicKey,
    SecretId,
};
use serde::Deserialize;
use sodiumoxide::crypto::sealedbox;
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod create_secret;
pub mod delete_secret;
pub mod get_secrets;
pub mod update_secret;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SecretsError {
    #[error("can't delete a secret with components connected: {0}")]
    CantDeleteSecret(SecretId),
    #[error("keypair error: {0}")]
    KeyPair(#[from] dal::KeyPairError),
    #[error("changes not permitted on HEAD change set")]
    NotPermittedOnHead,
    #[error("secret error: {0}")]
    Secret(#[from] dal::SecretError),
    #[error("secret definition view error: {0}")]
    SecretDefinitionView(#[from] dal::SecretDefinitionViewError),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error("definition not found for secret: {0}")]
    SecretWithInvalidDefinition(SecretId),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

pub type SecretsResult<T> = Result<T, SecretsError>;

#[derive(Deserialize, ToSchema)]
pub struct SecretV1RequestPath {
    #[schema(value_type = String)]
    pub secret_id: SecretId,
}

impl IntoResponse for SecretsError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl From<JsonRejection> for SecretsError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                SecretsError::Validation(format!("Invalid JSON data format: {rejection}"))
            }
            JsonRejection::JsonSyntaxError(_) => {
                SecretsError::Validation(format!("Invalid JSON syntax: {rejection}"))
            }
            JsonRejection::MissingJsonContentType(_) => SecretsError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => SecretsError::Validation(format!("JSON validation error: {rejection}")),
        }
    }
}

impl crate::service::v1::common::ErrorIntoResponse for SecretsError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            SecretsError::Secret(dal::SecretError::SecretNotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            SecretsError::SecretNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SecretsError::NotPermittedOnHead => (StatusCode::BAD_REQUEST, self.to_string()),
            SecretsError::SecretWithInvalidDefinition(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            SecretsError::CantDeleteSecret(_) => (StatusCode::CONFLICT, self.to_string()),
            SecretsError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_secrets::get_secrets))
        .route("/", post(create_secret::create_secret))
        .nest(
            "/:secret_id",
            Router::new()
                .route("/", delete(delete_secret::delete_secret))
                .route("/", put(update_secret::update_secret)),
        )
}

pub async fn encrypt_message(message: HashMap<String, String>, public_key: &PublicKey) -> Vec<u8> {
    sodiumoxide::init().expect("Failed to initialize sodiumoxide");

    let box_public_key = public_key.public_key();

    let serialized = serialize_message(&message);
    sealedbox::seal(&serialized, box_public_key)
}

fn serialize_message(message: &HashMap<String, String>) -> Vec<u8> {
    serde_json::to_vec(message).expect("Failed to serialize message")
}

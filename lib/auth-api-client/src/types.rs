use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AuthApiClientError {
    #[error("Auth token is not in Bearer format")]
    AuthTokenNotBearer,
    #[error("Request error: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Upload error: {0}")]
    Upload(String),
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type AuthApiResult<T> = Result<T, AuthApiClientError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoamiResponse {
    pub id: Ulid,
    pub auth0_id: String,
    pub auth0_details: serde_json::Value,
    pub nickname: String,
    pub email: String,
    pub email_verified: bool,
    pub first_name: String,
    pub last_name: String,
    pub picture_url: String,
    pub discord_username: String,
    pub github_username: String,
    pub onboarding_details: serde_json::Value,
    pub agreed_tos_version: String,
    pub needs_tos_update: bool,
}

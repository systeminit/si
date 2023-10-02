use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum IndexClientError {
    #[error("Request error: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Upload error: {0}")]
    Upload(String),
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type IndexClientResult<T> = Result<T, IndexClientError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleRejectionResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModulePromotedResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinsDetailsResponse {
    pub modules: Vec<ModuleDetailsResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleDetailsResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_user_id: String,
    pub owner_display_name: Option<String>,
    pub metadata: serde_json::Value,
    pub latest_hash: String,
    pub latest_hash_created_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuncMetadata {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

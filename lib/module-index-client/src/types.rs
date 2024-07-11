use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum IndexClientError {
    #[error("Deserialization error: {0}")]
    Deserialization(serde_json::Error),
    #[error("Request error: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(serde_json::Error),
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
    pub schema_id: Option<String>,
    pub past_hashes: Option<Vec<String>>,
}

impl ModuleDetailsResponse {
    pub fn schema_id(&self) -> Option<Ulid> {
        self.schema_id
            .as_deref()
            .and_then(|schema_id| Ulid::from_string(schema_id).ok())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuncMetadata {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraMetadata {
    pub version: String,
    pub schemas: Vec<String>,
    pub funcs: Vec<FuncMetadata>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListLatestModulesRequest {
    pub hashes: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListLatestModulesResponse {
    pub modules: Vec<LatestModuleResponse>,
}

/// This struct is nearly the same as the [`ModuleDetailsResponse`], but it does not include `past_hashes` since the
/// data is unneeded and requires additional query logic.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LatestModuleResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_user_id: String,
    pub owner_display_name: Option<String>,
    pub metadata: serde_json::Value,
    pub latest_hash: String,
    pub latest_hash_created_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub schema_id: Option<String>,
}

impl LatestModuleResponse {
    pub fn schema_id(&self) -> Option<Ulid> {
        self.schema_id
            .as_deref()
            .and_then(|schema_id| Ulid::from_string(schema_id).ok())
    }
}

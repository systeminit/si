use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

// TODO Move this to a substitute of si-pkg
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceExport {
    V0(WorkspaceExportContentV0),
}

impl WorkspaceExport {
    pub fn new(content: WorkspaceExportContentV0) -> Self {
        WorkspaceExport::V0(content)
    }

    #[allow(dead_code)]
    // This function should always return the latest version, updating the contents if necessary
    pub fn into_latest(self) -> WorkspaceExportContentV0 {
        let WorkspaceExport::V0(export) = self;
        export
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExportContentV0 {
    // We store changesets keyed by the cs id they depend on, so we can import in the right order
    pub change_sets: HashMap<Ulid, Vec<WorkspaceExportChangeSetV0>>,
    pub content_store_values: Vec<u8>,
    pub metadata: WorkspaceExportMetadataV0,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExportChangeSetV0 {
    pub id: Ulid,
    pub name: String,
    pub base_change_set_id: Option<Ulid>,
    pub workspace_snapshot_serialized_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceExportMetadataV0 {
    pub name: String,
    pub version: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub default_change_set: Ulid,
    pub default_change_set_base: Ulid,
    pub workspace_pk: Ulid,
    pub workspace_name: String,
}

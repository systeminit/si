use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub const MODULE_BUNDLE_FIELD_NAME: &str = "module_bundle";
pub const MODULE_BASED_ON_HASH_FIELD_NAME: &str = "based_on_hash";
pub const MODULE_SCHEMA_ID_FIELD_NAME: &str = "schema_id";
pub const MODULE_SCHEMA_VARIANT_ID_FIELD_NAME: &str = "schema_variant_id";
pub const MODULE_SCHEMA_VARIANT_VERSION_FIELD_NAME: &str = "schema_variant_version";
pub const MODULE_IS_PRIVATE_SCOPED_FIELD_NAME: &str = "is_private_scoped";

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListModulesResponse {
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
    pub schema_variant_id: Option<String>,
    pub schema_variant_version: Option<String>,
    pub structural_hash: Option<String>,
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
pub struct ListLatestModulesResponse {
    pub modules: Vec<LatestModuleResponse>,
}

/// This struct is nearly the same as the [`ModuleDetailsResponse`], but it does not include `past_hashes` since the
/// data is unneeded and requires additional query logic.
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
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

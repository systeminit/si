use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_user_id: String,
    pub owner_display_name: Option<String>,
    pub latest_hash: String,
    pub latest_hash_created_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

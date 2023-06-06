use serde::{Serialize, Deserialize};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    pub id: Ulid,
    pub name: String,
    pub description: Option<String>,
    pub owner_user_id: String,
    pub owner_display_name: Option<String>,
    pub latest_hash: String,
    pub latest_hash_created_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

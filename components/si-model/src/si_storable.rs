use chrono::prelude::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiStorable {
    pub type_name: String,
    pub object_id: String,
    pub billing_account_id: String,
    pub organization_id: String,
    pub workspace_id: String,
    pub tenant_ids: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SimpleStorable {
    pub type_name: String,
    pub object_id: String,
    pub billing_account_id: String,
    pub tenant_ids: Vec<String>,
    pub deleted: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MinimalStorable {
    pub type_name: String,
    pub object_id: String,
    pub deleted: bool,
}

use serde::Serialize;
use si_events::{AuthenticationMethod, ChangeSetId, UserPk};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    pub title: String,
    pub user_id: Option<UserPk>,
    pub user_email: Option<String>,
    pub user_name: Option<String>,
    pub kind: String,
    pub entity_type: String,
    pub entity_name: String,
    pub timestamp: String,
    pub change_set_id: Option<ChangeSetId>,
    pub change_set_name: Option<String>,
    pub metadata: serde_json::Value,
    pub authentication_method: AuthenticationMethod,
}

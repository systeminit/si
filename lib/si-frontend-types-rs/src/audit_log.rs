use serde::{Deserialize, Serialize};
use si_events::{audit_log::AuditLogKind, Actor};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    pub actor: Actor,
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
    pub kind: AuditLogKind,
    pub timestamp: String,
    pub origin_ip_address: Option<String>,
    pub workspace_id: String,
    pub workspace_name: Option<String>,
    pub change_set_id: Option<String>,
    pub change_set_name: Option<String>,
}

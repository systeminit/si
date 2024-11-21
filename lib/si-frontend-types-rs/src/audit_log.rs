use serde::Serialize;
use si_events::{ChangeSetId, UserPk};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    /// The title of the [`AuditLog`]. It will likely be combined with the `entity_type` to make a full display name.
    pub title: String,
    /// The identifier of the user. If this is empty, it is the system user.
    pub user_id: Option<UserPk>,
    /// The email of the user.
    pub user_email: Option<String>,
    /// The name of the user.
    pub user_name: Option<String>,
    /// The [kind](AuditLogKing) of the [`AuditLog`] (converted into a string because enum discriminants are not
    /// serializable).
    pub kind: String,
    /// The entity type.
    pub entity_type: String,
    /// The entity name.
    pub entity_name: String,
    /// The timestamp in ISO RFC 3339 format (converted into a string).
    pub timestamp: String,
    /// The identifier of the change set, which will only be empty for actions taken outside of the workspace.
    pub change_set_id: Option<ChangeSetId>,
    /// The name of the change set.
    pub change_set_name: Option<String>,
    /// Serialized version of [`AuditLogMetadata`](si_events::audit_log::AuditLogMetadata), which is an
    /// untagged version of the specific [`AuditLogKind`](si_events::audit_log::AuditLogKind).
    pub metadata: serde_json::Value,
}

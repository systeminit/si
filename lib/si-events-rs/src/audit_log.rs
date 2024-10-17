use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Actor, ChangeSetId, WorkspacePk};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct AuditLog {
    pub actor: Actor,
    pub service: AuditLogService,
    pub kind: AuditLogKind,
    pub timestamp: String,
    pub origin_ip_address: Option<String>,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum AuditLogService {
    AuthApi,
    Pinga,
    Rebaser,
    Sdf,
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum AuditLogKind {
    CreateComponent,
    DeleteComponent,
    PerformedRebase,
    RanAction,
    RanDependentValuesUpdate,
    UpdatePropertyEditorValue,
}

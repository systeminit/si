use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use crate::Actor;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct AuditLog {
    pub actor: Actor,
    pub kind: AuditLogKind,
    pub timestamp: String,
    pub origin_ip_address: Option<String>,
}

#[remain::sorted]
#[derive(
    Clone,
    Debug,
    Deserialize,
    Serialize,
    Eq,
    PartialEq,
    Hash,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
)]
pub enum AuditLogKind {
    CreateComponent,
    DeleteComponent,
    PerformedRebase,
    RanAction,
    RanDependentValuesUpdate,
    UpdatePropertyEditorValue,
}

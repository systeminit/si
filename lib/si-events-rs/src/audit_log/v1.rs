use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use crate::Actor;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct AuditLogV1 {
    pub actor: Actor,
    pub kind: AuditLogKindV1,
    pub timestamp: String,
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
pub enum AuditLogKindV1 {
    CreateComponent,
    DeleteComponent,
    PerformRebase,
    RunAction,
    RunComputeValidations,
    RunDependentValuesUpdate,
    UpdatePropertyEditorValue,
}

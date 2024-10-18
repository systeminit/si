use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
pub enum AuditLogService {
    AuthApi,
    Pinga,
    Rebaser,
    Sdf,
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

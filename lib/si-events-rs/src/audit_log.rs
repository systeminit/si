use serde::{Deserialize, Serialize};

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum AuditLogService {
    AuthApi,
    Pinga,
    Rebaser,
    Sdf,
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum AuditLogKind {
    CreateComponent,
    DeleteComponent,
    PerformedRebase,
    RanDependentValuesUpdate,
    UpdatePropertyEditorValue,
}

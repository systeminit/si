use serde::{Deserialize, Serialize};

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

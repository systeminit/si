use serde::{Deserialize, Serialize};

#[remain::sorted]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceKind {
    ChangeSetList,
    ChangeSetRecord,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Reference {
    pub kind: ReferenceKind,
    pub id: String,
    pub checksum: String,
}

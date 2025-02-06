use serde::{Deserialize, Serialize};

#[remain::sorted]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceKind {
    ChangeSetList,
    ChangeSetRecord,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReferenceId<T>(pub T)
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Reference<T>
where
    T: Serialize + Eq + PartialEq + Clone + std::fmt::Debug,
{
    pub kind: ReferenceKind,
    pub id: ReferenceId<T>,
    pub checksum: String,
}

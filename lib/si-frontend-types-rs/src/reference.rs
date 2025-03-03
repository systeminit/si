use serde::{Deserialize, Serialize};
use si_events::workspace_snapshot::{Checksum, ChecksumHasher};

use crate::{checksum::FrontendChecksum, object::FrontendObject};

#[remain::sorted]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Deserialize,
    Serialize,
    strum::Display,
    strum::EnumIter,
)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceKind {
    ChangeSetList,
    ChangeSetRecord,
    MvIndex,
    View,
    ViewList,
}

// Why the #[serde(bound...)] stuff? Well..
//
// See: https://github.com/serde-rs/serde/issues/964#issuecomment-364326970
// See: https://serde.rs/attr-bound.html
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ReferenceId<T>(#[serde(bound(deserialize = "T: Deserialize<'de>"))] pub T)
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display;

impl<T> std::fmt::Display for ReferenceId<T>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Reference<T>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
{
    pub kind: ReferenceKind,
    pub id: ReferenceId<T>,
    pub checksum: String,
}

#[derive(
    Clone, Debug, Eq, PartialEq, Serialize, Deserialize, si_frontend_types_macros::FrontendChecksum,
)]
pub struct IndexReference {
    pub kind: ReferenceKind,
    pub id: String,
    pub checksum: String,
}

impl<T> From<Reference<T>> for IndexReference
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
{
    fn from(value: Reference<T>) -> Self {
        Self {
            kind: value.kind,
            id: value.id.to_string(),
            checksum: value.checksum,
        }
    }
}

impl From<FrontendObject> for IndexReference {
    fn from(value: FrontendObject) -> Self {
        IndexReference {
            kind: value.kind,
            id: value.id.to_string(),
            checksum: value.checksum.to_string(),
        }
    }
}

impl<T> FrontendChecksum for ReferenceId<T>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
{
    fn checksum(&self) -> Checksum {
        todo!()
    }
}

impl FrontendChecksum for ReferenceKind {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(self.to_string().as_bytes());
        hasher.finalize()
    }
}

impl<T> FrontendChecksum for Reference<T>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
{
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.kind).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.id.to_string()).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.checksum.to_string()).as_bytes());

        hasher.finalize()
    }
}

pub trait Refer<T>: FrontendChecksum
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display + FrontendChecksum,
{
    fn reference(&self) -> Reference<T> {
        Reference {
            kind: self.reference_kind(),
            id: self.reference_id(),
            checksum: FrontendChecksum::checksum(self).to_string(),
        }
    }

    fn reference_kind(&self) -> ReferenceKind;
    fn reference_id(&self) -> ReferenceId<T>;
}

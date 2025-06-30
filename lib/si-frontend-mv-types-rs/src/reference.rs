use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::{
    Checksum,
    ChecksumHasher,
};

use crate::{
    checksum::FrontendChecksum,
    object::FrontendObject,
};

pub mod weak;

pub use weak::WeakReference;

#[remain::sorted]
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    strum::IntoStaticStr,
)]
#[serde(rename_all = "PascalCase")]
pub enum ReferenceKind {
    ActionPrototypeViewList,
    ActionViewList,
    AttributeTree,
    ChangeSetList,
    ChangeSetRecord,
    Component,
    ComponentInList,
    ComponentList,
    IncomingConnections,
    IncomingConnectionsList,
    ManagementConnections,
    MvIndex,
    SchemaMembers,
    SchemaVariant,
    SchemaVariantCategories,
    View,
    ViewComponentList,
    ViewList,
}

// Why the #[serde(bound...)] stuff? Well..
//
// See: https://github.com/serde-rs/serde/issues/964#issuecomment-364326970
// See: https://serde.rs/attr-bound.html
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, PartialOrd, Ord)]
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

#[derive(Clone, Debug, Serialize, Deserialize, si_frontend_mv_types_macros::FrontendChecksum)]
pub struct IndexReference {
    pub kind: String,
    pub id: String,
    pub checksum: String,
}

impl<T> From<Reference<T>> for IndexReference
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
{
    fn from(value: Reference<T>) -> Self {
        Self {
            kind: value.kind.to_string(),
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

// When you manually implement `std::cmp::Ord`, you are also supposed to manually
// implement `std::cmp::PartialOrd`, `std::cmp::PartialEq`, and `std::cmp::Eq`.
impl std::cmp::Ord for IndexReference {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.kind.cmp(&other.kind) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.id.cmp(&other.id) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.checksum.cmp(&other.checksum)
    }
}

impl std::cmp::PartialOrd for IndexReference {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for IndexReference {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.id == other.id && self.checksum == other.checksum
    }
}

impl Eq for IndexReference {}

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

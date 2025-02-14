use serde::{Deserialize, Serialize};
use si_events::workspace_snapshot::{Checksum, ChecksumHasher};

use crate::checksum::FrontendChecksum;

#[remain::sorted]
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, strum::Display, strum::EnumIter,
)]
#[serde(rename_all = "camelCase")]
pub enum ReferenceKind {
    ChangeSetList,
    ChangeSetRecord,
    View,
    ViewList,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReferenceId<T>(pub T)
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Reference<T>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
{
    pub kind: ReferenceKind,
    pub id: ReferenceId<T>,
    pub checksum: Checksum,
}

impl<T> std::fmt::Display for ReferenceId<T>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

// impl<T> FrontendChecksum for ReferenceId<T>
// where
//     T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
// {
//     fn checksum(&self) -> Checksum {
//         todo!()
//     }
// }

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
            checksum: FrontendChecksum::checksum(self),
        }
    }

    fn reference_kind(&self) -> ReferenceKind;
    fn reference_id(&self) -> ReferenceId<T>;
}

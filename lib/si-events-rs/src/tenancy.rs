use std::str::FromStr;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WorkspacePk(Ulid);

impl WorkspacePk {
    pub fn new() -> WorkspacePk {
        WorkspacePk(Ulid::new())
    }

    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl Default for WorkspacePk {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for WorkspacePk {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

impl From<ulid::Ulid> for WorkspacePk {
    fn from(value: ulid::Ulid) -> Self {
        Self(value)
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ChangeSetPk(Ulid);

impl ChangeSetPk {
    pub fn new() -> ChangeSetPk {
        ChangeSetPk(Ulid::new())
    }

    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl Default for ChangeSetPk {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ulid::Ulid> for ChangeSetPk {
    fn from(value: ulid::Ulid) -> Self {
        Self(value)
    }
}

impl FromStr for ChangeSetPk {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tenancy {
    pub change_set_pk: ChangeSetPk,
    pub workspace_pk: WorkspacePk,
}

impl Tenancy {
    pub fn new(workspace_pk: WorkspacePk, change_set_pk: ChangeSetPk) -> Self {
        Tenancy {
            change_set_pk,
            workspace_pk,
        }
    }
}

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use ulid::{Ulid, ULID_LEN};

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WorkspacePk(Ulid);

impl WorkspacePk {
    pub fn new() -> WorkspacePk {
        WorkspacePk(Ulid::new())
    }

    pub fn array_to_str<'buf>(&self, buf: &'buf mut [u8; ULID_LEN]) -> &'buf mut str {
        self.0.array_to_str(buf)
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

impl fmt::Display for WorkspacePk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ChangeSetId(Ulid);

impl ChangeSetId {
    pub fn new() -> ChangeSetId {
        ChangeSetId(Ulid::new())
    }

    pub fn array_to_str<'buf>(&self, buf: &'buf mut [u8; ULID_LEN]) -> &'buf mut str {
        self.0.array_to_str(buf)
    }

    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl Default for ChangeSetId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ChangeSetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<ulid::Ulid> for ChangeSetId {
    fn from(value: ulid::Ulid) -> Self {
        Self(value)
    }
}

impl FromStr for ChangeSetId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tenancy {
    pub change_set_id: ChangeSetId,
    pub workspace_pk: WorkspacePk,
}

impl Tenancy {
    pub fn new(workspace_pk: WorkspacePk, change_set_id: ChangeSetId) -> Self {
        Tenancy {
            change_set_id,
            workspace_pk,
        }
    }
}

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

const DEFAULT_WEB_EVENT_VERSION: u64 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebEvent {
    version: u64,
    workspace_pk: WorkspacePk,
    change_set_pk: ChangeSetPk,
    payload: WebEventPayload,
}

impl WebEvent {
    pub fn workspace_pk(&self) -> WorkspacePk {
        self.workspace_pk
    }

    pub fn change_set_pk(&self) -> ChangeSetPk {
        self.change_set_pk
    }

    pub fn payload(&self) -> &WebEventPayload {
        &self.payload
    }

    pub fn change_set_written(workspace_pk: WorkspacePk, change_set_pk: ChangeSetPk) -> Self {
        Self {
            version: DEFAULT_WEB_EVENT_VERSION,
            workspace_pk,
            change_set_pk,
            payload: WebEventPayload::ChangeSetWritten(change_set_pk),
        }
    }
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WebEventPayload {
    ChangeSetWritten(ChangeSetPk),
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WorkspacePk(Ulid);

impl WorkspacePk {
    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl FromStr for WorkspacePk {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ChangeSetPk(Ulid);

impl ChangeSetPk {
    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl FromStr for ChangeSetPk {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

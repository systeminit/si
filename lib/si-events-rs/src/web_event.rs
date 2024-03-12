use serde::{Deserialize, Serialize};

use crate::{tenancy::ChangeSetPk, tenancy::WorkspacePk};

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

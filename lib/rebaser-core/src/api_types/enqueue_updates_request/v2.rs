use serde::{Deserialize, Serialize};
use si_events::{rebase_batch_address::RebaseBatchAddress, ChangeSetId, WorkspacePk};

use crate::RequestId;

use super::EnqueueUpdatesRequest;

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueUpdatesRequestV2 {
    pub id: RequestId,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub updates_address: RebaseBatchAddress,
    pub from_change_set_id: Option<ChangeSetId>,
    pub audit_log_session_id: Option<si_events::ulid::Ulid>,
}

impl From<EnqueueUpdatesRequestV2> for EnqueueUpdatesRequest {
    fn from(value: EnqueueUpdatesRequestV2) -> Self {
        Self::V2(value)
    }
}

use naxum_api_types::RequestId;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ChangeSetId,
    EventSessionId,
    RebaseBatchAddressKind,
    WorkspacePk,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueUpdatesRequestV3 {
    pub id: RequestId,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub updates_address: RebaseBatchAddressKind,
    pub from_change_set_id: Option<ChangeSetId>,
    pub event_session_id: Option<EventSessionId>,
}

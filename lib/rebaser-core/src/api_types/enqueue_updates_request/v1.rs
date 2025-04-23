use naxum_api_types::RequestId;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ChangeSetId,
    WorkspacePk,
    rebase_batch_address::RebaseBatchAddress,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueUpdatesRequestV1 {
    pub id: RequestId,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub updates_address: RebaseBatchAddress,
    pub from_change_set_id: Option<ChangeSetId>,
}

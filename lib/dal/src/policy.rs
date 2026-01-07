use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    DalContext,
    WsEvent,
    WsEventResult,
    WsPayload,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PolicyUploadedPayload {}

impl WsEvent {
    pub async fn policy_uploaded(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::PolicyUploaded(PolicyUploadedPayload {})).await
    }
}

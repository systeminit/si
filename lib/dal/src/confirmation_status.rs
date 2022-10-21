use crate::{ConfirmationPrototypeId, DalContext, WsEvent, WsPayload};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ConfirmationStatus {
    Running,
    Success,
    Failure,
    Pending,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationStatusUpdate {
    confirmation_prototype_id: ConfirmationPrototypeId,
    status: ConfirmationStatus,
}

impl WsEvent {
    pub fn confirmation_status_update(
        ctx: &DalContext,
        confirmation_prototype_id: ConfirmationPrototypeId,
        status: ConfirmationStatus,
    ) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::ConfirmationStatusUpdate(ConfirmationStatusUpdate {
                confirmation_prototype_id,
                status,
            }),
        )
    }
}

use serde::{Deserialize, Serialize};

use crate::{ChangeSetId, DalContext, SecretId, WsEvent, WsEventResult, WsPayload};

#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecretCreatedPayload {
    secret_id: SecretId,
    change_set_id: ChangeSetId,
}

#[allow(missing_docs)]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecretUpdatedPayload {
    secret_id: SecretId,
    change_set_id: ChangeSetId,
}

impl WsEvent {
    #[allow(missing_docs)]
    pub async fn secret_created(ctx: &DalContext, secret_id: SecretId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SecretCreated(SecretCreatedPayload {
                secret_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    #[allow(missing_docs)]
    pub async fn secret_updated(ctx: &DalContext, secret_id: SecretId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SecretUpdated(SecretUpdatedPayload {
                secret_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }
}

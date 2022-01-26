use axum::Json;
use dal::{ComponentId, SystemId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceResponse {
    pub success: bool,
}

pub async fn sync_resource(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(_claim): Authorization,
    Json(_request): Json<SyncResourceRequest>,
) -> ComponentResult<Json<SyncResourceResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;

    // TODO

    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(SyncResourceResponse { success: true }))
}

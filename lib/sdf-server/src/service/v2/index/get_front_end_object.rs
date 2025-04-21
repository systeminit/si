use serde::{Deserialize, Serialize};

use axum::{
    Json,
    extract::{Path, Query},
};
use dal::{ChangeSet, ChangeSetId, WorkspacePk};

use crate::extract::{FriggStore, HandlerContext};

use super::{AccessBuilder, FrontEndObjectMeta, IndexError, IndexResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FrontendObjectRequest {
    pub kind: String,
    pub id: String,
    pub checksum: Option<String>,
}

pub async fn get_front_end_object(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Query(request): Query<FrontendObjectRequest>,
) -> IndexResult<Json<FrontEndObjectMeta>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let change_set = ChangeSet::get_by_id(&ctx, change_set_id).await?;

    let obj;
    if let Some(checksum) = request.checksum {
        obj = frigg
            .get_object(workspace_pk, &request.kind, &request.id, &checksum)
            .await?
            .ok_or(IndexError::IndexNotFound(workspace_pk, change_set_id))?;
    } else {
        obj = frigg
            .get_current_object(workspace_pk, change_set_id, &request.kind, &request.id)
            .await?
            .ok_or(IndexError::IndexNotFound(workspace_pk, change_set_id))?;
    }

    Ok(Json(FrontEndObjectMeta {
        workspace_snapshot_address: change_set.workspace_snapshot_address,
        front_end_object: obj,
    }))
}

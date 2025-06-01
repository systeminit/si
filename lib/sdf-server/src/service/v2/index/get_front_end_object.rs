use axum::{
    Json,
    extract::{
        Path,
        Query,
    },
};
use dal::{
    ChangeSetId,
    WorkspacePk,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    AccessBuilder,
    FrontEndObjectMeta,
    IndexError,
    IndexResult,
};
use crate::extract::{
    FriggStore,
    HandlerContext,
};

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
    let _ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let (checksum, address) = match frigg
        .get_index_pointer_value(workspace_pk, change_set_id)
        .await?
    {
        Some((index, _kv_revision)) => (index.index_checksum, index.snapshot_address),
        None => ("".to_string(), "".to_string()),
    };
    let obj;
    if let Some(checksum) = request.checksum {
        obj = frigg
            .get_object(workspace_pk, &request.kind, &request.id, &checksum)
            .await?
            .ok_or(IndexError::ItemWithChecksumNotFound(
                workspace_pk,
                change_set_id,
                request.kind,
            ))?;
    } else {
        obj = frigg
            .get_current_object(workspace_pk, change_set_id, &request.kind, &request.id)
            .await?
            .ok_or(IndexError::LatestItemNotFound(
                workspace_pk,
                change_set_id,
                request.kind,
            ))?;
    }

    Ok(Json(FrontEndObjectMeta {
        workspace_snapshot_address: address,
        index_checksum: checksum,
        front_end_object: obj,
    }))
}

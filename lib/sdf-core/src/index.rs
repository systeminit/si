use std::time::Duration;

use axum::response::{
    IntoResponse,
    Response,
};
use dal::{
    ChangeSetId,
    WorkspacePk,
};
use hyper::StatusCode;
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_mv_types::object::FrontendObject;
use si_frontend_types::FrontEndObjectRequest;
use thiserror::Error;
use tokio::task::JoinError;

use crate::api_error::ApiError;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FrontEndObjectMeta {
    pub workspace_snapshot_address: String,
    pub index_checksum: String,
    pub front_end_object: FrontendObject,
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum IndexError {
    #[error("semaphore acquire error: {0}")]
    Acquire(#[from] tokio::sync::AcquireError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("deserializing mv index data error: {0}")]
    DeserializingMvIndexData(#[source] serde_json::Error),
    #[error("edda client error: {0}")]
    EddaClient(#[from] edda_client::ClientError),
    #[error("frigg error: {0}")]
    Frigg(#[from] frigg::FriggError),
    #[error("index not found; workspace_id={0}, change_set_id={1}")]
    IndexNotFound(WorkspacePk, ChangeSetId),
    #[error("index not found after fresh rebuild (v1); workspace_id={0}, change_set_id={1}")]
    IndexNotFoundAfterFreshBuild(WorkspacePk, ChangeSetId),
    #[error("index not found after rebuild (v1); workspace_id={0}, change_set_id={1}")]
    IndexNotFoundAfterRebuild(WorkspacePk, ChangeSetId),
    #[error("item with checksum not found; workspace_id={0}, change_set_id={1}, kind={2}")]
    ItemWithChecksumNotFound(WorkspacePk, ChangeSetId, String),
    #[error("latest item not found; workspace_id={0}, change_set_id={1}, kind={2}")]
    LatestItemNotFound(WorkspacePk, ChangeSetId, String),
    #[error("tokio task join error: {0}")]
    TokioJoin(#[from] JoinError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("timed out when watching index with duration: {0:?}")]
    WatchIndexTimeout(Duration),
}

pub type IndexResult<T> = Result<T, IndexError>;

impl IntoResponse for IndexError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            IndexError::IndexNotFound(_, _)
            | IndexError::IndexNotFoundAfterFreshBuild(_, _)
            | IndexError::IndexNotFoundAfterRebuild(_, _)
            | IndexError::ItemWithChecksumNotFound(_, _, _)
            | IndexError::LatestItemNotFound(_, _, _) => StatusCode::NOT_FOUND,
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

pub async fn front_end_object_meta(
    frigg: &frigg::FriggStore,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    request: &FrontEndObjectRequest,
) -> IndexResult<FrontEndObjectMeta> {
    let (checksum, address) = match frigg
        .get_change_set_index_pointer_value(workspace_id, change_set_id)
        .await?
    {
        Some((index, _kv_revision)) => (index.index_checksum, index.snapshot_address),
        None => ("".to_string(), "".to_string()),
    };
    let obj;
    if let Some(checksum) = &request.checksum {
        obj = frigg
            .get_workspace_object(workspace_id, &request.kind, &request.id, checksum)
            .await?
            .ok_or_else(|| {
                IndexError::ItemWithChecksumNotFound(
                    workspace_id,
                    change_set_id,
                    request.kind.clone(),
                )
            })?;
    } else {
        obj = frigg
            .get_current_workspace_object(workspace_id, change_set_id, &request.kind, &request.id)
            .await?
            .ok_or_else(|| {
                IndexError::LatestItemNotFound(workspace_id, change_set_id, request.kind.clone())
            })?;
    }

    Ok(FrontEndObjectMeta {
        workspace_snapshot_address: address,
        index_checksum: checksum,
        front_end_object: obj,
    })
}

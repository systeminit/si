use std::time::Duration;

use axum::{
    Router,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        get,
        post,
    },
};
use dal::{
    ChangeSetId,
    WorkspacePk,
};
use futures_lite::StreamExt;
use hyper::StatusCode;
use sdf_core::api_error::ApiError;
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_mv_types::object::FrontendObject;
use telemetry::prelude::*;
use thiserror::Error;

use super::AccessBuilder;
use crate::AppState;

mod get_change_set_index;
mod get_front_end_object;
mod rebuild_change_set_index;

const WATCH_INDEX_TIMEOUT: Duration = Duration::from_secs(4);

#[remain::sorted]
#[derive(Error, Debug)]
pub enum IndexError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("deserializing mv index data error: {0}")]
    DeserializingMvIndexData(#[source] serde_json::Error),
    #[error("edda client error: {0}")]
    EddaClient(#[from] edda_client::ClientError),
    #[error("frigg error: {0}")]
    Frigg(#[from] frigg::FriggError),
    #[error("index not found; workspace_pk={0}, change_set_id={1}")]
    IndexNotFound(WorkspacePk, ChangeSetId),
    #[error("index not found after rebuild; workspace_pk={0}, change_set_id={1}")]
    IndexNotFoundAfterFreshBuild(WorkspacePk, ChangeSetId),
    #[error("index not found after rebuild; workspace_pk={0}, change_set_id={1}")]
    IndexNotFoundAfterRebuild(WorkspacePk, ChangeSetId),
    #[error("item with checksum not found; workspace_pk={0}, change_set_id={1}, kind={2}")]
    ItemWithChecksumNotFound(WorkspacePk, ChangeSetId, String),
    #[error("latest item not found; workspace_pk={0}, change_set_id={1}, kind={2}")]
    LatestItemNotFound(WorkspacePk, ChangeSetId, String),
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

pub fn v2_change_set_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_change_set_index::get_change_set_index))
        .route("/mjolnir", get(get_front_end_object::get_front_end_object))
        .route(
            "/rebuild",
            post(rebuild_change_set_index::rebuild_change_set_index),
        )
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FrontEndObjectMeta {
    workspace_snapshot_address: String,
    index_checksum: String,
    front_end_object: FrontendObject,
}

#[instrument(
    level = "info",
    name = "sdf.index.request_rebuild",
    skip_all,
    fields(
        si.edda_request.id = Empty
    )
)]
async fn request_rebuild(
    edda_client: &edda_client::EddaClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
) -> IndexResult<()> {
    let span = Span::current();
    let request_id = edda_client
        .rebuild_for_change_set(workspace_pk, change_set_id)
        .await?;
    span.record("si.edda_request.id", request_id.to_string());
    Ok(())
}

#[instrument(
    level = "info",
    name = "sdf.index.request_rebuild_and_watch",
    skip_all,
    fields(
        si.edda_request.id = Empty
    )
)]
pub async fn request_rebuild_and_watch(
    frigg: &frigg::FriggStore,
    edda_client: &edda_client::EddaClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
) -> IndexResult<bool> {
    let span = Span::current();
    let mut watch = frigg.watch_index(workspace_pk, change_set_id).await?;
    let request_id = edda_client
        .rebuild_for_change_set(workspace_pk, change_set_id)
        .await?;
    span.record("si.edda_request.id", request_id.to_string());

    let timeout = WATCH_INDEX_TIMEOUT;
    tokio::select! {
        _ = tokio::time::sleep(timeout) => {
            info!("timed out waiting for new index to be rebuilt");
            Ok(false)
        },
        _ = watch.next() => Ok(true)
    }
}

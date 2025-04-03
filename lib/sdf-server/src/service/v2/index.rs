use futures_lite::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dal::{ChangeSet, ChangeSetId, WorkspacePk, WorkspaceSnapshotAddress};
use hyper::StatusCode;
use si_frontend_types::{index::MvIndex, object::FrontendObject, reference::ReferenceKind};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    extract::{EddaClient, FriggStore, HandlerContext},
    service::ApiError,
    AppState,
};

use super::AccessBuilder;

const WATCH_INDEX_TIMEOUT: Duration = Duration::from_secs(30);

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
    #[error("invalid string for reference kind: {0}")]
    InvalidStringForReferenceKind(#[source] strum::ParseError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("timed out when watching index with duration: {0:?}")]
    WatchIndexTimeout(Duration),
}

pub type IndexResult<T> = Result<T, IndexError>;

impl IntoResponse for IndexError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            IndexError::IndexNotFound(_, _) => StatusCode::NOT_FOUND,
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

pub async fn get_workspace_index(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    Path(workspace_pk): Path<WorkspacePk>,
) -> IndexResult<Json<HashMap<ChangeSetId, Option<FrontendObject>>>> {
    let ctx = builder.build_head(access_builder).await?;

    let mut indexes = HashMap::new();
    for change_set in ChangeSet::list_active(&ctx).await? {
        let maybe_index = frigg.get_index(workspace_pk, change_set.id).await?;
        indexes.insert(change_set.id, maybe_index.map(|i| i.0));
    }

    Ok(Json(indexes))
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FrontEndObjectMeta {
    workspace_snapshot_address: WorkspaceSnapshotAddress,
    front_end_object: FrontendObject,
}

pub async fn get_change_set_index(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    EddaClient(edda_client): EddaClient,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> IndexResult<Json<FrontEndObjectMeta>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let change_set = ChangeSet::get_by_id(&ctx, change_set_id).await?;

    let index = match frigg.get_index(workspace_pk, change_set_id).await? {
        Some((index, _kv_revision)) => {
            let mv_index: MvIndex = serde_json::from_value(index.data.to_owned())
                .map_err(IndexError::DeserializingMvIndexData)?;

            // NOTE(nick,jacob): this may or may not be better suited for "edda". Let's trace this
            // to ensure that this stopgap solution does not bog the system down.
            let span = info_span!("sdf.index.get_change_set_index.implemented_kinds");
            let implemented_kinds = span.in_scope(|| {
                let mut implemented_kinds = HashSet::new();
                for index_ref in mv_index.mv_list {
                    let kind = ReferenceKind::try_from(index_ref.kind.as_str())
                        .map_err(IndexError::InvalidStringForReferenceKind)?;
                    if kind.is_revision_sensitive() {
                        implemented_kinds.insert(kind);
                    }
                }
                IndexResult::Ok(implemented_kinds)
            })?;

            if implemented_kinds == ReferenceKind::revision_sensitive() {
                index
            } else {
                info!(
                    "Index out of date for change_set {}; attempting full build",
                    change_set_id,
                );
                request_rebuild_and_watch(&frigg, &edda_client, workspace_pk, change_set_id)
                    .await?;
                frigg
                    .get_index(workspace_pk, change_set_id)
                    .await?
                    .map(|i| i.0)
                    .ok_or(IndexError::IndexNotFound(workspace_pk, change_set_id))?
            }
        }
        None => {
            info!(
                "Index not found for change_set {}; attempting full build",
                change_set_id,
            );
            request_rebuild_and_watch(&frigg, &edda_client, workspace_pk, change_set_id).await?;
            frigg
                .get_index(workspace_pk, change_set_id)
                .await?
                .map(|i| i.0)
                .ok_or(IndexError::IndexNotFound(workspace_pk, change_set_id))?
        }
    };

    Ok(Json(FrontEndObjectMeta {
        workspace_snapshot_address: change_set.workspace_snapshot_address,
        front_end_object: index,
    }))
}

#[instrument(
    level = "info",
    name = "sdf.index.get_change_set_index.request_rebuild_and_watch",
    skip_all
)]
async fn request_rebuild_and_watch(
    frigg: &frigg::FriggStore,
    edda_client: &edda_client::EddaClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
) -> IndexResult<()> {
    let mut watch = frigg.watch_index(workspace_pk, change_set_id).await?;
    edda_client
        .rebuild_for_change_set(workspace_pk, change_set_id)
        .await?;
    let timeout = WATCH_INDEX_TIMEOUT;
    tokio::select! {
        _ = tokio::time::sleep(timeout) => Err(IndexError::WatchIndexTimeout(timeout)),
        _ = watch.next() => Ok(())
    }
}

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

pub fn v2_workspace_routes() -> Router<AppState> {
    Router::new().route("/", get(get_workspace_index))
}

pub fn v2_change_set_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_change_set_index))
        .route("/mjolnir", get(get_front_end_object))
}

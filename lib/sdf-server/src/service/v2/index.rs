use std::collections::HashMap;

use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dal::{ChangeSet, ChangeSetId, WorkspacePk};
use hyper::StatusCode;
use si_frontend_types::object::FrontendObject;
use thiserror::Error;

use crate::{
    extract::{FriggStore, HandlerContext},
    service::ApiError,
    AppState,
};

use super::AccessBuilder;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum IndexError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("frigg error: {0}")]
    Frigg(#[from] frigg::FriggError),
    #[error("index not found; workspace_pk={0}, change_set_id={1}")]
    IndexNotFound(WorkspacePk, ChangeSetId),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
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

pub async fn get_change_set_index(
    FriggStore(frigg): FriggStore,
    Path(workspace_pk): Path<WorkspacePk>,
    Path(change_set_id): Path<ChangeSetId>,
) -> IndexResult<Json<FrontendObject>> {
    let index = frigg
        .get_index(workspace_pk, change_set_id)
        .await?
        .map(|i| i.0)
        .ok_or(IndexError::IndexNotFound(workspace_pk, change_set_id))?;

    Ok(Json(index))
}

pub fn v2_workspace_routes() -> Router<AppState> {
    Router::new().route("/", get(get_workspace_index))
}

pub fn v2_change_set_routes() -> Router<AppState> {
    Router::new().route("/", get(get_change_set_index))
}

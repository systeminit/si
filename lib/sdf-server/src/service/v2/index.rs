use std::time::Duration;

use axum::{
    Router,
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
use sdf_core::index::IndexResult;
use telemetry::prelude::*;

use super::AccessBuilder;
use crate::AppState;

mod get_change_set_index;
mod get_front_end_object;
mod rebuild_change_set_index;

const WATCH_INDEX_TIMEOUT: Duration = Duration::from_secs(4);

pub fn v2_change_set_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_change_set_index::get_change_set_index))
        .route("/mjolnir", get(get_front_end_object::get_front_end_object))
        .route(
            "/multi_mjolnir",
            post(get_front_end_object::get_multiple_front_end_objects),
        )
        .route(
            "/rebuild",
            post(rebuild_change_set_index::rebuild_change_set_index),
        )
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

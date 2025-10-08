use axum::{
    Json,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};
use dal::WorkspacePk;
use sdf_core::index::FrontEndObjectMeta;
use telemetry::prelude::*;

use super::{
    AccessBuilder,
    IndexResult,
};
use crate::extract::{
    EddaClient,
    FriggStore,
    HandlerContext,
};

// async fn request_rebuild_and_watch(
//     frigg: &frigg::FriggStore,
//     edda_client: &edda_client::EddaClient,
//     workspace_pk: WorkspacePk,
//     change_set_id: ChangeSetId,
// ) -> IndexResult<bool> {
//     let span = Span::current();
//     let mut watch = frigg.watch_deployment_index().await?;
//     let request_id = edda_client
//         .rebuild_for_change_set(workspace_pk, change_set_id)
//         .await?;
//     span.record("si.edda_request.id", request_id.to_string());

//     let timeout = WATCH_INDEX_TIMEOUT;
//     tokio::select! {
//         _ = tokio::time::sleep(timeout) => {
//             info!("timed out waiting for new index to be rebuilt");
//             Ok(false)
//         },
//         _ = watch.next() => Ok(true)
//     }
// }

pub async fn get_deployment_index(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    EddaClient(_edda_client): EddaClient,
    Path(_workspace_id): Path<WorkspacePk>,
) -> IndexResult<impl IntoResponse> {
    let _ctx = builder.build_head(access_builder).await?;
    let index = match frigg.get_deployment_index().await? {
        Some((index, _kv_revision)) => index,
        None => {
            info!("Index not found for deployment; attempting full build");
            // if !request_rebuild_and_watch(&frigg, &edda_client, workspace_pk, change_set_id).await?
            // {
            // Return 202 Accepted with the same response body if the build didn't succeed in time
            // to let the caller know the create succeeded, we're just waiting on downstream work
            return Ok((StatusCode::ACCEPTED, Json(None)));
            //}
            // frigg
            //     .get_deployment_index()
            //     .await?
            //     .map(|i| i.0)
            //     .ok_or(IndexError::DeploymentIndexNotFoundAfterFreshBuild())?
        }
    };

    Ok((
        StatusCode::OK,
        Json(Some(FrontEndObjectMeta {
            workspace_snapshot_address: index.clone().id,
            index_checksum: index.clone().checksum,

            front_end_object: index,
        })),
    ))
}

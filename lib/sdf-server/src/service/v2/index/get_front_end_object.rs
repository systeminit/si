use std::sync::Arc;

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
use sdf_core::index::{
    FrontEndObjectMeta,
    IndexError,
    front_end_object_meta,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_types::FrontEndObjectRequest;
use telemetry::prelude::*;
use tokio::{
    sync::Semaphore,
    task::JoinSet,
};

use super::{
    AccessBuilder,
    IndexResult,
};
use crate::extract::{
    FriggStore,
    HandlerContext,
};

pub async fn get_front_end_object(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Query(request): Query<FrontEndObjectRequest>,
) -> IndexResult<Json<FrontEndObjectMeta>> {
    let _ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    Ok(Json(
        front_end_object_meta(&frigg, workspace_pk, change_set_id, &request).await?,
    ))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MultipleFrontEndObjectRequest {
    pub requests: Vec<FrontEndObjectRequest>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MultipleFrontEndObjectResponse {
    successful: Vec<FrontEndObjectMeta>,
    failed: Vec<FrontEndObjectRequest>,
}

const BULK_CONCURRENCY_LIMIT: usize = 200;

pub async fn get_multiple_front_end_objects(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<MultipleFrontEndObjectRequest>,
) -> IndexResult<Json<MultipleFrontEndObjectResponse>> {
    let concurrency_control = Arc::new(Semaphore::new(BULK_CONCURRENCY_LIMIT));

    let _ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let mut successful = Vec::new();
    let mut failed = Vec::new();

    let mut join_set = JoinSet::new();

    for object_request in request.requests {
        let frigg_clone = frigg.clone();
        let sem_clone = concurrency_control.clone();
        join_set.spawn(async move {
            let _permit = sem_clone.acquire().await?;
            Ok::<
                (
                    Result<FrontEndObjectMeta, IndexError>,
                    FrontEndObjectRequest,
                ),
                IndexError,
            >((
                front_end_object_meta(&frigg_clone, workspace_pk, change_set_id, &object_request)
                    .await,
                object_request,
            ))
        });
    }

    while let Some(join_result) = join_set.join_next().await {
        match join_result?? {
            (Ok(meta), _) => successful.push(meta),
            (Err(error), object_request) => {
                error!(?error);
                failed.push(object_request);
            }
        };
    }

    Ok(Json(MultipleFrontEndObjectResponse { successful, failed }))
}

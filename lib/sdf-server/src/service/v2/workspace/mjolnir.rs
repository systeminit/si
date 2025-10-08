use std::sync::Arc;

use axum::{
    Json,
    extract::{
        Path,
        Query,
    },
};
use dal::WorkspacePk;
use frigg::FriggStore;
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_mv_types::object::FrontendObject;
use si_frontend_types::FrontEndObjectRequest;
use telemetry::prelude::*;
use tokio::{
    sync::Semaphore,
    task::JoinSet,
};

use super::AccessBuilder;
use crate::{
    extract::{
        FriggStore as FriggStoreExtractor,
        HandlerContext,
    },
    service::v2::workspace::{
        WorkspaceAPIError,
        WorkspaceAPIResult,
    },
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MultipleFrontEndObjectRequest {
    pub requests: Vec<FrontEndObjectRequest>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MultipleFrontEndObjectResponse {
    successful: Vec<FrontendObject>,
    failed: Vec<FrontEndObjectRequest>,
}

pub async fn get_front_end_object(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStoreExtractor(frigg): FriggStoreExtractor,
    Path(workspace_pk): Path<WorkspacePk>,
    Query(request): Query<FrontEndObjectRequest>,
) -> WorkspaceAPIResult<Json<FrontendObject>> {
    let _ctx = builder.build_head(access_builder).await?;

    let obj = frigg
        .get_current_deployment_object(&request.kind, &request.id)
        .await?;
    match obj {
        Some(o) => Ok(Json(o)),
        None => Err(WorkspaceAPIError::LatestItemNotFound(
            workspace_pk,
            request.kind,
            request.id,
        )),
    }
}

const BULK_CONCURRENCY_LIMIT: usize = 200;

pub async fn get_multiple_front_end_objects(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStoreExtractor(frigg): FriggStoreExtractor,
    Path(_workspace_pk): Path<WorkspacePk>,
    Json(request): Json<MultipleFrontEndObjectRequest>,
) -> WorkspaceAPIResult<Json<MultipleFrontEndObjectResponse>> {
    let concurrency_control = Arc::new(Semaphore::new(BULK_CONCURRENCY_LIMIT));

    let _ctx = builder.build_head(access_builder).await?;

    let mut successful = Vec::new();
    let mut failed = Vec::new();

    let mut join_set = JoinSet::new();

    let maybe_mv_index = frigg.get_deployment_index().await?.map(|r| r.0);
    let Some(mv_index_data) = maybe_mv_index else {
        return Err(WorkspaceAPIError::IndexNotFound);
    };
    let mv_list = Arc::new(FriggStore::mv_list_from_deployment_mv_index_version_data(
        mv_index_data.data,
    )?);
    for object_request in request.requests {
        let mv_list_clone = mv_list.clone();
        let frigg_clone = frigg.clone();
        let sem_clone = concurrency_control.clone();
        join_set.spawn(async move {
            let _permit = sem_clone.acquire().await?;
            Ok::<(Result<Option<FrontendObject>, _>, FrontEndObjectRequest), WorkspaceAPIError>((
                frigg_clone
                    .get_current_deployment_object_with_mvlist(
                        &object_request.kind,
                        &object_request.id,
                        mv_list_clone.as_ref(),
                    )
                    .await
                    .map_err(Into::<WorkspaceAPIError>::into),
                object_request,
            ))
        });
    }

    while let Some(join_result) = join_set.join_next().await {
        match join_result?? {
            (Ok(Some(obj)), _) => successful.push(obj),
            (Ok(None), object_request) => {
                error!("Not found {:?}", object_request);
                failed.push(object_request);
            }
            (Err(error), object_request) => {
                error!(?error);
                failed.push(object_request);
            }
        };
    }

    Ok(Json(MultipleFrontEndObjectResponse { successful, failed }))
}

use serde::{Deserialize, Serialize};
use si_data::{NatsConn, PgPool};
use si_model::{Resource, ResourceError, Veritech};

use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceRequest {
    pub entity_id: String,
    pub system_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceReply {
    pub resource: Option<Resource>,
}

pub async fn get_resource(
    pg: PgPool,
    token: String,
    request: GetResourceRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "resourceDal", "getResource").await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.system_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.entity_id,
        &claim.billing_account_id,
    )
    .await?;

    let maybe_resource =
        Resource::get_by_entity_and_system(&txn, request.entity_id, request.system_id)
            .await
            .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;

    let reply = GetResourceReply {
        resource: maybe_resource,
    };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceRequest {
    pub entity_id: String,
    pub system_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SyncResourceReply {
    pub started: bool,
}

pub async fn sync_resource(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: SyncResourceRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "resourceDal", "syncResource").await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.system_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.entity_id,
        &claim.billing_account_id,
    )
    .await?;

    let resource = Resource::get_by_entity_and_system(&txn, &request.entity_id, &request.system_id)
        .await
        .map_err(HandlerError::from)?
        .ok_or_else(|| ResourceError::NoResource(request.entity_id, request.system_id))
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;

    resource
        .sync(pg, nats_conn, veritech)
        .await
        .map_err(HandlerError::from)?;

    let reply = SyncResourceReply { started: true };
    Ok(warp::reply::json(&reply))
}

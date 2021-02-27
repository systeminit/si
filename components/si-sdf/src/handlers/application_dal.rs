use serde::{Deserialize, Serialize};
use si_data::{NatsConn, PgPool};
use si_model::{application, ApplicationListEntry, Veritech};

use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationRequest {
    pub application_name: String,
    pub workspace_id: String,
}

pub type CreateApplicationReply = ApplicationListEntry;

pub async fn create_application(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: CreateApplicationRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "applicationDal", "createApplication").await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;

    let application_list_entry = application::create(
        pg.clone(),
        nats_conn.clone(),
        &nats,
        &veritech,
        &request.application_name,
        &request.workspace_id,
    )
    .await
    .map_err(HandlerError::from)?;

    nats.commit().await.map_err(HandlerError::from)?;

    let reply: CreateApplicationReply = application_list_entry;

    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationsRequest {
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationsReply {
    pub list: Vec<ApplicationListEntry>,
}

pub async fn list_applications(
    pg: PgPool,
    token: String,
    request: ListApplicationsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "applicationDal", "listApplications").await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;

    let list = application::list(&txn, request.workspace_id)
        .await
        .map_err(HandlerError::from)?;

    let reply = ListApplicationsReply { list };
    Ok(warp::reply::json(&reply))
}

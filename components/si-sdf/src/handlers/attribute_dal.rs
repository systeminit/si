use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use serde::{Deserialize, Serialize};
use si_data::PgPool;
use si_model::{application, ApplicationEntities};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectListRequest {
    pub workspace_id: String,
    pub application_id: String,
    pub change_set_id: Option<String>,
}

pub type GetObjectListReply = ApplicationEntities;

pub async fn get_object_list(
    pg: PgPool,
    token: String,
    request: GetObjectListRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getObjectList").await?;
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
        &request.application_id,
        &claim.billing_account_id,
    )
    .await?;
    if let Some(change_set_id) = request.change_set_id.as_ref() {
        validate_tenancy(
            &txn,
            "change_sets",
            &change_set_id,
            &claim.billing_account_id,
        )
        .await?;
    }

    let reply = application::all_entities(
        &txn,
        &request.application_id,
        request.change_set_id.as_ref(),
    )
    .await
    .map_err(HandlerError::from)?;

    Ok(warp::reply::json(&reply))
}

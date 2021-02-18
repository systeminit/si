use crate::{
    data::{NatsConn, PgPool},
    handlers::{authenticate, authorize, validate_tenancy, HandlerError},
    models::{EdgeKind, Schematic},
    veritech::Veritech,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationSystemSchematicRequest {
    pub workspace_id: String,
    pub root_object_id: String,
    pub change_set_id: Option<String>,
    pub system_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationSystemSchematicReply {
    schematic: Schematic,
}

pub async fn get_application_system_schematic(
    pg: PgPool,
    token: String,
    request: GetApplicationSystemSchematicRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(
        &txn,
        &claim.user_id,
        "schematicDal",
        "getApplicationSystemSchematic",
    )
    .await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "systems",
        &request.system_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.root_object_id,
        &claim.billing_account_id,
    )
    .await?;

    let schematic = Schematic::get(
        &txn,
        &request.root_object_id,
        &request.workspace_id,
        &request.system_id,
        request.change_set_id,
        vec![EdgeKind::Configures],
    )
    .await
    .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;

    let reply = GetApplicationSystemSchematicReply { schematic };
    Ok(warp::reply::json(&reply))
}

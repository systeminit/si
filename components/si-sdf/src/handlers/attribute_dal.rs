use crate::{
    data::{NatsConn, PgPool},
    handlers::{authenticate, authorize, validate_tenancy, HandlerError, LabelListItem},
    models::{Edge, EdgeKind, Entity, Node, Schematic},
    veritech::Veritech,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectListRequest {
    pub workspace_id: String,
    pub application_id: String,
    pub change_set_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectListReply {
    pub object_list: Vec<LabelListItem>,
}

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

    let mut object_list: Vec<LabelListItem> = Vec::new();

    let root_entity = Entity::get_any(&txn, &request.application_id)
        .await
        .map_err(HandlerError::from)?;

    let successors =
        Edge::all_successor_edges_by_object_id(&txn, &EdgeKind::Configures, &root_entity.id)
            .await
            .map_err(HandlerError::from)?;

    for edge in successors.into_iter() {
        if let Some(entity) = Entity::get_relevant_projection_or_head(
            &txn,
            &edge.head_vertex.object_id,
            request.change_set_id.clone(),
        )
        .await
        .map_err(HandlerError::from)?
        {
            object_list.push(LabelListItem {
                label: entity.name,
                value: edge.head_vertex.object_id,
            });
        }
    }

    let reply = GetObjectListReply { object_list };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntityRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub change_set_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntityReply {
    pub entity: Entity,
}

pub async fn get_entity(
    pg: PgPool,
    token: String,
    request: GetEntityRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getEntity").await?;
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
        &request.entity_id,
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

    let entity =
        Entity::get_relevant_projection_or_head(&txn, &request.entity_id, request.change_set_id)
            .await
            .map_err(HandlerError::from)?
            .ok_or(HandlerError::NotFound)?;

    let reply = GetEntityReply { entity };
    Ok(warp::reply::json(&reply))
}

use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use serde::{Deserialize, Serialize};
use si_data::{NatsConn, PgPool};
use si_model::{
    application,
    entity::diff::{diff_for_props, Diffs},
    ApplicationEntities, Entity, LabelList, LabelListItem, Qualification, Schematic, SchematicKind,
    Veritech,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntityListRequest {
    pub workspace_id: String,
    pub application_id: String,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
}

pub type GetEntityListReply = ApplicationEntities;

pub async fn get_entity_list(
    pg: PgPool,
    token: String,
    request: GetEntityListRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getEntityList").await?;
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
    if let Some(edit_session_id) = request.edit_session_id.as_ref() {
        validate_tenancy(
            &txn,
            "edit_sessions",
            &edit_session_id,
            &claim.billing_account_id,
        )
        .await?;
    }

    let mut reply = application::all_entities(
        &txn,
        &request.application_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    .map_err(HandlerError::from)?;

    reply.entity_list.insert(
        0,
        LabelListItem {
            value: "".to_string(),
            label: "".to_string(),
        },
    );

    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntityRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntityReply {
    pub entity: Entity,
    pub diff: Diffs,
    pub qualifications: Vec<Qualification>,
}

pub async fn get_entity(
    pg: PgPool,
    token: String,
    request: GetEntityRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getEntityList").await?;
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
    if let Some(edit_session_id) = request.edit_session_id.as_ref() {
        validate_tenancy(
            &txn,
            "edit_sessions",
            &edit_session_id,
            &claim.billing_account_id,
        )
        .await?;
    }

    let entity = Entity::for_head_or_change_set_or_edit_session(
        &txn,
        &request.entity_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    .map_err(|_| HandlerError::InvalidContext)?;

    let diff = match Entity::for_diff(
        &txn,
        &request.entity_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    {
        Ok(lhs) => diff_for_props(&lhs, &entity).map_err(HandlerError::from)?,
        Err(_e) => diff_for_props(&entity, &entity).map_err(HandlerError::from)?,
    };

    let qualifications: Vec<Qualification> = Qualification::for_head_or_change_set_or_edit_session(
        &txn,
        &request.entity_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = GetEntityReply {
        entity,
        diff,
        qualifications,
    };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetInputLabelsRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub input_name: String,
    pub schematic_kind: SchematicKind,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetInputLabelsReply {
    pub items: LabelList,
}

pub async fn get_input_labels(
    pg: PgPool,
    token: String,
    request: GetInputLabelsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getInputLabels").await?;
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
    if let Some(edit_session_id) = request.edit_session_id.as_ref() {
        validate_tenancy(
            &txn,
            "edit_sessions",
            &edit_session_id,
            &claim.billing_account_id,
        )
        .await?;
    }

    Entity::for_head_or_change_set_or_edit_session(
        &txn,
        &request.entity_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    .map_err(|_| HandlerError::InvalidContext)?;

    let schematic = Schematic::get_by_schematic_kind(
        &txn,
        &request.schematic_kind,
        &request.entity_id,
        request.change_set_id.clone(),
        request.edit_session_id.clone(),
    )
    .await
    .map_err(HandlerError::from)?;

    let mut items: LabelList = vec![];
    for edge in schematic.edges.values() {
        if edge.head_vertex.object_id == request.entity_id
            && edge.head_vertex.socket == request.input_name
        {
            let schematic_node = match schematic.nodes.get(&edge.tail_vertex.node_id) {
                Some(schematic_node) => schematic_node,
                None => continue,
            };
            items.push(LabelListItem {
                label: format!(
                    "{}: {}",
                    &schematic_node.object.entity_type, &schematic_node.object.name,
                ),
                value: schematic_node.object.id.clone(),
            });
        }
    }

    let reply = GetInputLabelsReply { items };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntityRequest {
    pub workspace_id: String,
    pub entity: Entity,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub system_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntityReply {
    pub entity: Entity,
    pub diff: Diffs,
    pub qualifications: Vec<Qualification>,
    pub label: LabelListItem,
}

pub async fn update_entity(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: UpdateEntityRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "saveEntity").await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "change_sets",
        &request.change_set_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "edit_sessions",
        &request.edit_session_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.entity.id,
        &claim.billing_account_id,
    )
    .await?;
    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    // These are going to get committed toot-suite. Probably dumb to have them
    // here, but.. in a hurry

    let mut entity = request.entity;
    entity
        .update_entity_for_edit_session(
            &pg,
            &nats_conn,
            &veritech,
            &request.change_set_id,
            &request.edit_session_id,
        )
        .await
        .map_err(HandlerError::from)?;

    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let diff = match Entity::for_diff(
        &txn,
        &entity.id,
        Some(&request.change_set_id),
        Some(&request.edit_session_id),
    )
    .await
    {
        Ok(lhs) => diff_for_props(&lhs, &entity).map_err(HandlerError::from)?,
        Err(_e) => diff_for_props(&entity, &entity).map_err(HandlerError::from)?,
    };

    let label = LabelListItem {
        label: entity.name.clone(),
        value: entity.id.clone(),
    };

    let qualifications: Vec<Qualification> = Qualification::for_head_or_change_set_or_edit_session(
        &txn,
        &entity.id,
        Some(&request.change_set_id),
        Some(&request.edit_session_id),
    )
    .await
    .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    entity
        .check_qualifications_for_edit_session(
            &pg,
            &nats_conn,
            &veritech,
            request.system_id,
            &request.change_set_id,
            &request.edit_session_id,
        )
        .await
        .map_err(HandlerError::from)?;

    let reply = UpdateEntityReply {
        entity,
        diff,
        qualifications,
        label,
    };
    Ok(warp::reply::json(&reply))
}

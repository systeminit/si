use crate::data::{NatsConn, PgPool};
use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use crate::models::{ChangeSet, EditSession, Entity, Node, NodeKind, Resource, System};
use crate::veritech::Veritech;
use serde::{Deserialize, Serialize};

pub const APPLICATION_DAL_LIST_APPLICATIONS: &str =
    include_str!("../data/queries/application_dal_list_applications.sql");

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetCounts {
    open: i32,
    closed: i32,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServiceWithResources {
    service: Entity,
    resources: Vec<Resource>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationListEntry {
    pub application: Entity,
    pub systems: Vec<System>,
    pub services_with_resources: Vec<ServiceWithResources>,
    pub change_set_counts: ChangeSetCounts,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationRequest {
    pub application_name: String,
    pub workspace_id: String,
    pub system_id: String,
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
    validate_tenancy(
        &txn,
        "systems",
        &request.system_id,
        &claim.billing_account_id,
    )
    .await?;
    txn.commit().await.map_err(HandlerError::from)?;

    let mut cs_conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let cs_txn = cs_conn.transaction().await.map_err(HandlerError::from)?;
    let mut change_set = ChangeSet::new(&cs_txn, &nats, None, request.workspace_id.clone())
        .await
        .map_err(HandlerError::from)?;
    let edit_session = EditSession::new(
        &cs_txn,
        &nats,
        None,
        change_set.id.clone(),
        request.workspace_id.clone(),
    )
    .await
    .map_err(HandlerError::from)?;
    cs_txn.commit().await.map_err(HandlerError::from)?;

    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let application_node = Node::new(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        Some(request.application_name),
        NodeKind::Entity,
        "application",
        &request.workspace_id,
        &change_set.id,
        &edit_session.id,
        Some(vec![request.system_id]),
    )
    .await
    .map_err(HandlerError::from)?;
    txn.commit().await.map_err(HandlerError::from)?;

    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, false, None)
        .await
        .map_err(HandlerError::from)?;
    let application = application_node
        .get_head_object_entity(&txn)
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply: CreateApplicationReply = CreateApplicationReply {
        application,
        systems: vec![],
        services_with_resources: vec![],
        change_set_counts: ChangeSetCounts { open: 0, closed: 1 },
    };

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

    let rows = txn
        .query(APPLICATION_DAL_LIST_APPLICATIONS, &[&request.workspace_id])
        .await
        .map_err(HandlerError::from)?;

    let mut list = Vec::new();
    for row in rows.into_iter() {
        let json: serde_json::Value = row.try_get("application").map_err(HandlerError::from)?;
        let application: Entity = serde_json::from_value(json).map_err(HandlerError::from)?;
        list.push(ApplicationListEntry {
            application,
            systems: vec![],
            services_with_resources: vec![],
            change_set_counts: ChangeSetCounts { open: 0, closed: 1 },
        });
    }

    let reply = ListApplicationsReply { list };
    Ok(warp::reply::json(&reply))
}

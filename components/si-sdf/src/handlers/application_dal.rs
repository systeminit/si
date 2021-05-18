use serde::{Deserialize, Serialize};
use si_data::{NatsConn, PgPool};
use si_model::{
    application, ApplicationListEntry, Entity, Veritech, Workflow, WorkflowContext, WorkflowRun,
    Workspace,
};

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeployServicesRequest {
    pub workspace_id: String,
    pub system_id: String,
    pub application_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeployServicesReply {
    pub workflow_run: WorkflowRun,
}

pub async fn deploy_services(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: DeployServicesRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "applicationDal", "deployServices").await?;
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
    validate_tenancy(
        &txn,
        "entities",
        &request.system_id,
        &claim.billing_account_id,
    )
    .await?;

    let entity = Entity::for_head(&txn, &request.application_id)
        .await
        .map_err(HandlerError::from)?;
    let system = Entity::for_head(&txn, &request.system_id)
        .await
        .map_err(HandlerError::from)?;
    let workspace = Workspace::get(&txn, &request.workspace_id)
        .await
        .map_err(HandlerError::from)?;

    let workflow_name = Workflow::entity_and_action_name_to_workflow_name(&entity, "deploy");

    let ctx = WorkflowContext {
        dry_run: false,
        entity: Some(entity),
        system: Some(system),
        selection: vec![],
        strategy: None,
        fail_if_missing: None,
        inputs: None,
        args: None,
        output: None,
        store: None,
        workspace,
    };

    let workflow_run = Workflow::get_by_name(&txn, workflow_name)
        .await
        .map_err(HandlerError::from)?
        .invoke(&pg, &nats_conn, &veritech, ctx)
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = DeployServicesReply { workflow_run };

    Ok(warp::reply::json(&reply))
}

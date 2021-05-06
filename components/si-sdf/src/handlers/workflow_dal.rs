use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use serde::{Deserialize, Serialize};
use si_data::{NatsConn, PgPool};
use si_model::workflow::WorkflowRunListItem;
use si_model::{Entity, Veritech, Workflow, WorkflowContext, WorkflowRun, Workspace};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunActionRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub action_name: String,
    pub system_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunActionReply {
    pub workflow_run: WorkflowRun,
}

pub async fn run_action(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: RunActionRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "workflowDal", "runAction").await?;
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
    validate_tenancy(
        &txn,
        "entities",
        &request.system_id,
        &claim.billing_account_id,
    )
    .await?;

    let entity = Entity::for_head(&txn, &request.entity_id)
        .await
        .map_err(HandlerError::from)?;
    let system = Entity::for_head(&txn, &request.system_id)
        .await
        .map_err(HandlerError::from)?;
    let workspace = Workspace::get(&txn, &request.workspace_id)
        .await
        .map_err(HandlerError::from)?;

    let workflow_name =
        Workflow::entity_and_action_name_to_workflow_name(&entity, &request.action_name);

    let ctx = WorkflowContext {
        dry_run: true,
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

    let reply = RunActionReply { workflow_run };

    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListActionRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub system_id: String,
    pub action_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListActionReply {
    pub workflow_runs: Vec<WorkflowRunListItem>,
}

pub async fn list_action(
    pg: PgPool,
    token: String,
    request: ListActionRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "workflowDal", "listAction").await?;
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
    validate_tenancy(
        &txn,
        "entities",
        &request.system_id,
        &claim.billing_account_id,
    )
    .await?;

    let workflow_runs = WorkflowRun::list_actions(
        &txn,
        &request.entity_id,
        &request.system_id,
        &request.workspace_id,
        request.action_name.as_ref(),
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = ListActionReply { workflow_runs };

    Ok(warp::reply::json(&reply))
}

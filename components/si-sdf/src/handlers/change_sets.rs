use crate::data::{NatsConn, PgPool};
use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use crate::models::change_set::{
    ChangeSet, ChangeSetParticipant, CreateReply, CreateRequest, ExecuteReply, PatchOps,
    PatchReply, PatchRequest,
};
use crate::models::ops::OpEntityAction;
use crate::models::{Event, ListRequest, Node, PageToken, Query};
use crate::page_secret_key;
use crate::veritech::Veritech;

use tracing::trace;

pub async fn create(
    pg: PgPool,
    nats_conn: NatsConn,
    token: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    authorize(&txn, &claim.user_id, "changeSet", "create").await?;

    let change_set = ChangeSet::new(&txn, &nats, request.name, request.workspace_id)
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = CreateReply { item: change_set };
    Ok(warp::reply::json(&reply))
}

pub async fn patch(
    change_set_id: String,
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: PatchRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(
        &txn,
        "change_sets",
        &change_set_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    authorize(&txn, &claim.user_id, "changeSet", "patch").await?;

    let reply = match request.op {
        PatchOps::Execute(execute_request) => {
            trace!("executing changeset");
            let mut change_set: ChangeSet = ChangeSet::get(&txn, &change_set_id)
                .await
                .map_err(HandlerError::from)?;
            let impacted_ids = change_set
                .execute(
                    &pg,
                    &txn,
                    &nats_conn,
                    &nats,
                    &veritech,
                    execute_request.hypothetical,
                    None,
                )
                .await
                .map_err(HandlerError::from)?;
            PatchReply::Execute(ExecuteReply {
                item_ids: impacted_ids,
            })
        }
        PatchOps::ExecuteWithAction(execute_with_action_request) => {
            validate_tenancy(
                &txn,
                "nodes",
                &execute_with_action_request.node_id,
                &claim.billing_account_id,
            )
            .await?;
            validate_tenancy(
                &txn,
                "systems",
                &execute_with_action_request.system_id,
                &claim.billing_account_id,
            )
            .await?;
            validate_tenancy(
                &txn,
                "edit_sessions",
                &execute_with_action_request.edit_session_id,
                &claim.billing_account_id,
            )
            .await?;

            let mut change_set: ChangeSet = ChangeSet::get(&txn, &change_set_id)
                .await
                .map_err(HandlerError::from)?;

            // Create an event if the action is a "deploy"
            let event_id = if execute_with_action_request.action == "deploy" {
                let event = Event::change_set_execute(&pg, &nats_conn, &change_set, None)
                    .await
                    .map_err(HandlerError::from)?;
                Some(event.id)
            } else {
                None
            };

            trace!("appending action on change set");
            let node_id = execute_with_action_request.node_id;
            let entity_id = Node::get(&txn, &node_id)
                .await
                .map_err(HandlerError::from)?
                .get_object_id();

            OpEntityAction::new(
                &txn,
                &nats,
                entity_id,
                execute_with_action_request.action,
                execute_with_action_request.system_id,
                request.workspace_id,
                change_set_id,
                execute_with_action_request.edit_session_id,
            )
            .await
            .map_err(HandlerError::from)?;

            trace!("executing changeset");
            let impacted_ids = change_set
                .execute(
                    &pg,
                    &txn,
                    &nats_conn,
                    &nats,
                    &veritech,
                    false,
                    event_id.as_deref(),
                )
                .await
                .map_err(HandlerError::from)?;
            PatchReply::Execute(ExecuteReply {
                item_ids: impacted_ids,
            })
        }
    };

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    Ok(warp::reply::json(&reply))
}

pub async fn list_participants(
    pg: PgPool,
    token: String,
    request: ListRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "change_set_participants", "list").await?;

    let query = if let Some(query) = request.query {
        Some(Query::from_url_string(query).map_err(HandlerError::from)?)
    } else {
        None
    };

    let page_token = if let Some(page_token) = request.page_token {
        Some(PageToken::unseal(&page_token, page_secret_key()).map_err(HandlerError::from)?)
    } else {
        None
    };

    let reply = ChangeSetParticipant::list(
        &txn,
        &claim.billing_account_id,
        query,
        request.page_size,
        request.order_by,
        request.order_by_direction,
        page_token,
    )
    .await
    .map_err(HandlerError::from)?;

    Ok(warp::reply::json(&reply))
}

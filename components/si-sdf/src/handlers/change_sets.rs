use crate::data::{Connection, Db};
use tracing::trace;

use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models::change_set::{
    ChangeSet, CreateReply, CreateRequest, ExecuteReply, PatchOps, PatchReply, PatchRequest,
};
use crate::models::ops::OpEntityAction;
use crate::models::{Event, Node};

pub async fn create(
    db: Db,
    nats: Connection,
    token: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "changeSet",
        "create",
    )
    .await?;

    let change_set = ChangeSet::new(
        &db,
        &nats,
        request.name,
        claim.billing_account_id,
        request.organization_id,
        request.workspace_id,
        claim.user_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = CreateReply { item: change_set };
    Ok(warp::reply::json(&reply))
}

#[tracing::instrument(level = "info")]
pub async fn patch(
    change_set_id: String,
    db: Db,
    nats: Connection,
    token: String,
    request: PatchRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "changeSet",
        "patch",
    )
    .await?;

    let reply = match request.op {
        PatchOps::Execute(execute_request) => {
            trace!("executing changeset");
            let mut change_set: ChangeSet =
                ChangeSet::get(&db, &change_set_id, &claim.billing_account_id)
                    .await
                    .map_err(HandlerError::from)?;
            let impacted_ids = change_set
                .execute(&db, &nats, execute_request.hypothetical, None)
                .await
                .map_err(HandlerError::from)?;
            PatchReply::Execute(ExecuteReply {
                item_ids: impacted_ids,
            })
        }
        PatchOps::ExecuteWithAction(execute_with_action_request) => {
            let mut change_set: ChangeSet =
                ChangeSet::get(&db, &change_set_id, &claim.billing_account_id)
                    .await
                    .map_err(HandlerError::from)?;

            // Create an event if the action is a "deploy"
            let event_id = if execute_with_action_request.action == "deploy" {
                let event = Event::change_set_execute(&db, &nats, &change_set, None)
                    .await
                    .map_err(HandlerError::from)?;
                Some(event.id)
            } else {
                None
            };

            trace!("appending action on change set");
            let node_id = execute_with_action_request.node_id;
            let entity_id = Node::get(&db, &node_id, &claim.billing_account_id)
                .await
                .map_err(HandlerError::from)?
                .get_object_id(&db)
                .await
                .map_err(HandlerError::from)?;

            OpEntityAction::new(
                db.clone(),
                nats.clone(),
                entity_id,
                execute_with_action_request.action,
                execute_with_action_request.system_id,
                claim.billing_account_id,
                request.organization_id,
                request.workspace_id,
                change_set_id,
                execute_with_action_request.edit_session_id,
                claim.user_id,
            )
            .await
            .map_err(HandlerError::from)?;

            trace!("executing changeset");
            let impacted_ids = change_set
                .execute(&db, &nats, false, event_id.as_deref())
                .await
                .map_err(HandlerError::from)?;
            PatchReply::Execute(ExecuteReply {
                item_ids: impacted_ids,
            })
        }
    };
    Ok(warp::reply::json(&reply))
}

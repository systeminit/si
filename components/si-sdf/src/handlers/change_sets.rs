use crate::data::{Connection, Db};

use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models::change_set::{
    ChangeSet, CreateReply, CreateRequest, ExecuteReply, PatchOps, PatchReply, PatchRequest,
};

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
            let mut change_set: ChangeSet =
                ChangeSet::get(&db, &change_set_id, &claim.billing_account_id)
                    .await
                    .map_err(HandlerError::from)?;
            let impacted_ids = change_set
                .execute(&db, &nats, execute_request.hypothetical)
                .await
                .map_err(HandlerError::from)?;
            PatchReply::Execute(ExecuteReply {
                item_ids: impacted_ids,
            })
        }
    };
    Ok(warp::reply::json(&reply))
}

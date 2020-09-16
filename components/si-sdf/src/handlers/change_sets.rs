use si_data::Db;

use crate::handlers::{authorize, HandlerError};
use crate::models::change_set::{
    ChangeSet, CreateReply, CreateRequest, ExecuteReply, PatchReply, PatchRequest,
};

pub async fn create(
    db: Db,
    user_id: String,
    billing_account_id: String,
    organization_id: String,
    workspace_id: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    authorize(&db, &user_id, &billing_account_id, "changeSet", "create").await?;

    let change_set = ChangeSet::new(
        &db,
        request.name,
        billing_account_id,
        organization_id,
        workspace_id,
        user_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = CreateReply { item: change_set };
    Ok(warp::reply::json(&reply))
}

pub async fn patch(
    change_set_id: String,
    db: Db,
    user_id: String,
    billing_account_id: String,
    _organization_id: String,
    _workspace_id: String,
    request: PatchRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    authorize(&db, &user_id, &billing_account_id, "changeSet", "patch").await?;

    let reply = match request {
        PatchRequest::Execute(execute_request) => {
            let mut change_set: ChangeSet =
                ChangeSet::get(&db, &change_set_id, &billing_account_id)
                    .await
                    .map_err(HandlerError::from)?;
            let impacted_ids = change_set
                .execute(&db, execute_request.hypothetical)
                .await
                .map_err(HandlerError::from)?;
            PatchReply::Execute(ExecuteReply {
                item_ids: impacted_ids,
            })
        }
    };
    Ok(warp::reply::json(&reply))
}

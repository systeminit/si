use si_data::Db;

use crate::handlers::{authorize, HandlerError};
use crate::models::change_set::{ChangeSet, CreateReply, CreateRequest};

pub async fn create(
    db: Db,
    user_id: String,
    billing_account_id: String,
    organization_id: String,
    workspace_id: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    authorize(&db, &user_id, &billing_account_id).await?;

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

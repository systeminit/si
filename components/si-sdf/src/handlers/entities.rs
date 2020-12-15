use crate::data::Db;
use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models::{entity, Entity};


pub async fn get(
    entity_id: String,
    db: Db,
    token: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "entities",
        "get",
    )
    .await?;

    let entities = Entity::get_all(&db, &entity_id)
        .await
        .map_err(HandlerError::from)?;

    let reply = entity::GetReply { items: entities };

    Ok(warp::reply::json(&reply))
}

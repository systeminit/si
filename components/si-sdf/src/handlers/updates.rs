use nats::asynk::Connection;
use warp::ws::Ws;

use crate::data::Db;
use crate::handlers::{authenticate, authorize};
use crate::models::websocket_run;

#[tracing::instrument(level = "trace", target = "nodes::create")]
pub async fn update(
    ws: Ws,
    db: Db,
    nats: Connection,
    token: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "updates",
        "receive",
    )
    .await?;

    ws.on_upgrade(move |websocket| websocket_run(websocket, nats, claim));

    Ok(warp::reply())
}

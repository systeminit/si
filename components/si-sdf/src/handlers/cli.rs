use nats::asynk::Connection;
use warp::ws::Ws;

use crate::cli::server::websocket_run;
use crate::data::Db;
use crate::handlers::{authenticate, authorize};
use crate::models::WebsocketToken;

pub async fn cli(
    ws: Ws,
    db: Db,
    nats: Connection,
    ws_token: WebsocketToken,
) -> Result<impl warp::reply::Reply, warp::reject::Rejection> {
    let token = ws_token.token;
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "cli",
        "call",
    )
    .await?;

    Ok(ws.on_upgrade(move |websocket| websocket_run(websocket, db, nats, claim)))
}

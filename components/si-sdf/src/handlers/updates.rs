use crate::handlers::{authenticate, authorize, HandlerError};
use crate::update::{websocket_run, WebsocketToken};
use si_data::{NatsConn, PgPool};
use warp::ws::Ws;

pub async fn update(
    ws: Ws,
    ws_token: WebsocketToken,
    pg: PgPool,
    nats_conn: NatsConn,
) -> Result<impl warp::reply::Reply, warp::reject::Rejection> {
    let token = ws_token.token;
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "updates", "receive").await?;
    txn.commit().await.map_err(HandlerError::from)?;

    Ok(ws.on_upgrade(move |websocket| websocket_run(websocket, pg, nats_conn, claim)))
}

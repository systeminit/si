use crate::cli::server::{websocket_run, WebsocketToken};
use crate::handlers::{authenticate_api_client, authorize_api_client, HandlerError};
use si_data::{NatsConn, PgPool};
use si_model::Veritech;
use warp::ws::Ws;

pub async fn cli(
    ws: Ws,
    ws_token: WebsocketToken,
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> Result<impl warp::reply::Reply, warp::reject::Rejection> {
    let token = ws_token.token;
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate_api_client(&txn, &token).await?;
    authorize_api_client(&txn, &claim.api_client_id, "cli", "call").await?;
    txn.commit().await.map_err(HandlerError::from)?;

    Ok(ws.on_upgrade(move |websocket| websocket_run(websocket, pg, nats_conn, veritech, claim)))
}

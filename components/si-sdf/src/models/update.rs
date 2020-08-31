use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};

use crate::data::{Connection, Db};
use crate::handlers::users::SiClaims;
use crate::models::{load_billing_account_model, load_data_model, UpdateClock};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebsocketToken {
    pub token: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    op: UpdateOp,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum UpdateOp {
    Model(serde_json::Value),
    LoadFinished,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoadData {
    pub workspace_id: String,
    pub update_clock: UpdateClock,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ControlOp {
    LoadData(LoadData),
    Stop(serde_json::Value),
}

pub async fn websocket_run(websocket: WebSocket, db: Db, nats: Connection, claim: SiClaims) {
    // Split the socket into a sender and receiver of messages.
    let (ws_tx, mut ws_rx) = websocket.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    //let (outbound_ws_tx, rx): (
    //    mpsc::UnboundedSender<Result<Message, warp::Error>>,
    //    mpsc::UnboundedReceiver<Result<Message, warp::Error>>,
    //) = mpsc::unbounded_channel();
    let (outbound_ws_tx, rx) = mpsc::unbounded_channel();

    // For debugging!
    let claim2 = claim.clone();
    tokio::task::spawn(rx.forward(ws_tx).map(move |result| {
        if let Err(e) = result {
            tracing::error!("websocket send error for {:#?}: {}", claim2, e);
        }
    }));

    let outbound_ws_tx_from_nats = outbound_ws_tx.clone();
    // Send matching data to the web socket
    match nats
        .subscribe(&format!("{}.>", &claim.billing_account_id))
        .await
    {
        Ok(mut sub) => {
            tokio::task::spawn(async move {
                while let Some(msg) = sub.next().await {
                    match serde_json::from_slice(&msg.data) {
                        Ok(data_json) => match serde_json::to_string(&UpdateOp::Model(data_json)) {
                            Ok(op_json) => {
                                match outbound_ws_tx_from_nats.send(Ok(Message::text(op_json))) {
                                    Ok(_) => (),
                                    Err(err) => tracing::error!("cannot send outbound op: {}", err),
                                }
                            }
                            Err(err) => {
                                tracing::error!("cannot serialize op as json: {}", err);
                            }
                        },
                        Err(err) => tracing::error!("bad data from nats: {} / {:#?}", err, msg),
                    }
                }
            });
        }
        Err(err) => tracing::error!("websocket error creating subscriber: {}", err),
    }

    // Listen to ControlOps from the websocket
    while let Some(control_op_msg_result) = ws_rx.next().await {
        match control_op_msg_result {
            Ok(control_op_msg) => {
                match serde_json::from_slice::<ControlOp>(&control_op_msg.into_bytes()) {
                    Ok(control_op) => match control_op {
                        ControlOp::Stop(value) => {
                            tracing::debug!("graceful stop: {}", value);
                            break;
                        }
                        ControlOp::LoadData(load_data) => {
                            tracing::debug!(?load_data, "loading data");
                            let mut results = match load_data_model(
                                &db,
                                load_data.workspace_id,
                                load_data.update_clock,
                            )
                            .await
                            {
                                Ok(results) => results,
                                Err(err) => {
                                    tracing::error!(?err, "cannot load data");
                                    continue;
                                }
                            };
                            let mut b_results =
                                match load_billing_account_model(&db, &claim.billing_account_id)
                                    .await
                                {
                                    Ok(b_results) => b_results,
                                    Err(err) => {
                                        tracing::error!(?err, "cannot load data");
                                        continue;
                                    }
                                };
                            results.append(&mut b_results);

                            for model in results.into_iter() {
                                match serde_json::to_string(&UpdateOp::Model(model)) {
                                    Ok(op_json) => {
                                        match outbound_ws_tx.send(Ok(Message::text(op_json))) {
                                            Ok(_) => (),
                                            Err(err) => {
                                                tracing::error!("cannot send outbound op: {}", err)
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        tracing::error!("cannot serialize op as json: {}", err);
                                    }
                                }
                            }
                            match serde_json::to_string(&UpdateOp::LoadFinished) {
                                Ok(op_json) => {
                                    match outbound_ws_tx.send(Ok(Message::text(op_json))) {
                                        Ok(_) => (),
                                        Err(err) => {
                                            tracing::error!("cannot send outbound op: {}", err)
                                        }
                                    }
                                }
                                Err(err) => {
                                    tracing::error!("cannot serialize op as json: {}", err);
                                }
                            }
                        }
                    },
                    Err(err) => {
                        tracing::error!("error deserializing control op: {}", err);
                    }
                }
            }
            Err(err) => {
                tracing::error!("client has departed: {}", err);
                break;
            }
        }
    }
}

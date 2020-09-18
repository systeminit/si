use crate::handlers::users::SiClaims;
use crate::models::UpdateClock;

use futures::{FutureExt, StreamExt};
use nats::asynk::Connection;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    op: UpdateOp,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum UpdateOp {
    Model(String),
}

pub async fn websocket_run(websocket: WebSocket, nats: Connection, claim: SiClaims) {
    // Split the socket into a sender and receive of messages.
    let (ws_tx, mut ws_rx) = websocket.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();

    // For debugging!
    let claim2 = claim.clone();
    tokio::task::spawn(rx.forward(ws_tx).map(move |result| {
        if let Err(e) = result {
            tracing::error!("websocket send error for {:#?}: {}", claim2, e);
        }
    }));

    // Consumer Web Socket
    //      * Sends us ControlOps
    //      * Receives UpdateOps
    //
    //  We should create some kind of shared data structure that gets
    //  updated. A seperate tokio task should manage the control
    //  ops, and then a seperate tokio task should send the updateops
    //  based on subscriptions given by the ops requested.
    //
    //  TODO: Your mission, should you choose to accept it: get the websocket implementation
    //  working, so we can get the models in the typescript side of SDF to be correct.

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Make an extra clone to give to our disconnection handler...
    //let users2 = users.clone();

    //// Every time the user sends a message, broadcast it to
    //// all other users...
    //while let Some(result) = user_ws_rx.next().await {
    //    let msg = match result {
    //        Ok(msg) => msg,
    //        Err(e) => {
    //            eprintln!("websocket error(uid={}): {}", my_id, e);
    //            break;
    //        }
    //    };
    //    user_message(my_id, msg, &users).await;
    //}

    //// user_ws_rx stream will keep processing as long as the user stays
    //// connected. Once they disconnect, then...
    //user_disconnected(my_id, &users2).await;
}

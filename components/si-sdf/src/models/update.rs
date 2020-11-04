use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, UnboundedSender};
use tracing::{error, trace, warn};
use warp::ws::{Message, WebSocket};

use crate::data::{Connection, Db};
use crate::handlers::users::SiClaims;
use crate::models::{
    key_pair::KeyPair, load_billing_account_model, load_data_model, secret::EncryptedSecret,
    PublicKey, Secret, UpdateClock,
};

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MessageStream {
    Continue,
    Finish,
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
        if let Err(err) = result {
            // Doesn't look like `warp::Error` can deref the inner error, which is rather sad. The
            // "Connection closed normall" 'error' appears to be safe and benign, so we'll ignore
            // this one and warn on all others
            match err.to_string().as_ref() {
                "Connection closed normally" => {
                    trace!("ws client send closed normally; err={:?}", err)
                }
                _ => warn!("ws client send error; err={}, claim={:?}", err, claim2),
            }
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
                                    Err(err) => error!("cannot send outbound op; err={:?}", err),
                                }
                            }
                            Err(err) => error!("cannot serialize op as json: {}", err),
                        },
                        Err(err) => error!("bad data from nats: {} / {:#?}", err, msg),
                    }
                }
            });
        }
        Err(err) => error!("websocket error creating subscriber: {}", err),
    }

    // Listen to ControlOps from the websocket
    while let Some(message_result) = ws_rx.next().await {
        match message_result {
            Ok(message) => {
                if MessageStream::Finish
                    == process_message(&db, &claim, &outbound_ws_tx, message.clone()).await
                {
                    break;
                }
            }
            Err(err) => {
                trace!("ws client poll error; err={:?}", err);
                break;
            }
        }
    }

    trace!("ws client connection closed, good bye");
}

async fn process_message(
    db: &Db,
    claim: &SiClaims,
    outbound_ws_tx: &UnboundedSender<Result<Message, warp::Error>>,
    message: Message,
) -> MessageStream {
    // This `warp::ws::Message` wraps a `tungstenite::protocol::Message` which is an enum, but
    // sadly the warp Message does not expose the underlying enum for pattern mataching. Instead
    // we'll have to iterate through all the variants by hand with if statements
    if message.is_text() {
        trace!("recv ws text msg, processing; message={:?}", message);

        match serde_json::from_slice::<ControlOp>(&message.into_bytes()) {
            Ok(control_op) => match control_op {
                ControlOp::Stop(value) => {
                    trace!("recv control op stop; value={:?}", &value);
                    return MessageStream::Finish;
                }
                ControlOp::LoadData(load_data) => {
                    trace!("recv control op load data; load_data={:?}", &load_data);
                    let mut results =
                        match load_data_model(db, load_data.workspace_id, load_data.update_clock)
                            .await
                        {
                            Ok(results) => results,
                            Err(err) => {
                                tracing::error!(?err, "cannot load data");
                                return MessageStream::Continue;
                            }
                        };
                    let mut b_results =
                        match load_billing_account_model(db, &claim.billing_account_id).await {
                            Ok(b_results) => b_results,
                            Err(err) => {
                                tracing::error!(?err, "cannot load data");
                                return MessageStream::Continue;
                            }
                        };
                    results.append(&mut b_results);

                    for model in results.into_iter() {
                        let model =
                            if let Some(type_name) = model["siStorable"]["typeName"].as_str() {
                                match type_name {
                                    "keyPair" => {
                                        let key_pair: KeyPair =
                                            serde_json::from_value(model.clone()).expect(
                                                "deserialize into KeyPair failed, \
                                                the document data is suspect",
                                            );
                                        serde_json::to_value(PublicKey::from(key_pair))
                                            .expect("serialize into PublicKey failed")
                                    }
                                    "secret" => {
                                        let secret: EncryptedSecret =
                                            serde_json::from_value(model.clone()).expect(
                                                "deserialize into EncryptedSecret failed, \
                                                the document data is suspect",
                                            );
                                        serde_json::to_value(Secret::from(secret))
                                            .expect("serialize into Secret failed")
                                    }
                                    _ => model,
                                }
                            } else {
                                model
                            };

                        match serde_json::to_string(&UpdateOp::Model(model)) {
                            Ok(op_json) => match outbound_ws_tx.send(Ok(Message::text(op_json))) {
                                Ok(_) => (),
                                Err(err) => tracing::error!("cannot send outbound op: {}", err),
                            },
                            Err(err) => {
                                tracing::error!("cannot serialize op as json: {}", err);
                            }
                        }
                    }
                    match serde_json::to_string(&UpdateOp::LoadFinished) {
                        Ok(op_json) => match outbound_ws_tx.send(Ok(Message::text(op_json))) {
                            Ok(_) => (),
                            Err(err) => tracing::error!("cannot send outbound op: {}", err),
                        },
                        Err(err) => {
                            tracing::error!("cannot serialize op as json: {}", err);
                        }
                    }
                }
            },
            Err(err) => {
                tracing::error!("error deserializing control op: {}", err);
            }
        };
    } else if message.is_close() {
        trace!("recv ws close msg, skipping; message={:?}", message);
    } else if message.is_ping() {
        // Pings are automatically ponged via tungstenite so if we receive this message, there is
        // nothing left to do
        trace!("recv ws ping msg, skipping; message={:?}", message);
    } else if message.is_pong() {
        trace!("recv ws pong msg, skipping; message={:?}", message);
    } else if message.is_binary() {
        warn!(
            "recv ws binary message which is not expected (text only), skipping; message={:#x?}",
            message.as_bytes()
        );
    } else {
        // If we trigger this error, then the underlying `tungstenite::protocol::Message` likely
        // has a new variant that we are not not handling explicitly
        error!(
            "recv ws msg of unknown type, likely a new underlying variant, \
            programmer intervention required; message={:?}",
            message
        );
    }

    MessageStream::Continue
}

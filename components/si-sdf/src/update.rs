use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use si_data::{NatsConn, PgPool};
use si_model::{EncryptedSecret, KeyPair, PublicKey, Secret, SiClaims};
use thiserror::Error;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

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

pub async fn websocket_run(websocket: WebSocket, pg: PgPool, nats_conn: NatsConn, claim: SiClaims) {
    // Split the socket into a sender and receiver of messages.
    let (ws_tx, mut ws_rx) = websocket.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    //let (outbound_ws_tx, rx): (
    //    mpsc::UnboundedSender<Result<Message, warp::Error>>,
    //    mpsc::UnboundedReceiver<Result<Message, warp::Error>>,
    //) = mpsc::unbounded_channel();
    let (outbound_ws_tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);

    // For debugging!
    let claim2 = claim.clone();
    tokio::task::spawn(rx.forward(ws_tx).map(move |result| {
        if let Err(err) = result {
            // Doesn't look like `warp::Error` can deref the inner error, which is rather sad. The
            // "Connection closed normall" 'error' appears to be safe and benign, so we'll ignore
            // this one and warn on all others
            match err.to_string().as_ref() {
                "Connection closed normally" => {
                    dbg!("ws client send closed normally; err={:?}", &err);
                }
                _ => {
                    dbg!("ws client send error; err={}, claim={:?}", &err, &claim2);
                }
            }
        }
    }));

    let outbound_ws_tx_from_nats = outbound_ws_tx.clone();
    // Send matching data to the web socket
    match nats_conn
        .subscribe(&format!("{}.>", &claim.billing_account_id))
        .await
    {
        Ok(sub) => {
            tokio::task::spawn(async move {
                while let Some(msg) = sub.next().await {
                    match serde_json::from_slice(&msg.data) {
                        Ok(data_json) => match serde_json::to_string(&UpdateOp::Model(data_json)) {
                            Ok(op_json) => {
                                match outbound_ws_tx_from_nats.send(Ok(Message::text(op_json))) {
                                    Ok(_) => (),
                                    Err(err) => {
                                        dbg!("cannot send outbound op; other side likely disconnected");
                                        dbg!(&err);
                                        break;
                                    }
                                }
                            }
                            Err(err) => {
                                dbg!("cannot serialize op as json: {}", err);
                            }
                        },
                        Err(err) => {
                            dbg!("bad data from nats: {} / {:#?}", err, msg);
                        }
                    }
                }
            });
        }
        Err(err) => {
            dbg!("websocket error creating subscriber: {}", err);
        }
    }

    // Listen to ControlOps from the websocket
    while let Some(message_result) = ws_rx.next().await {
        match message_result {
            Ok(message) => {
                if MessageStream::Finish
                    == process_message(&pg, &claim, &outbound_ws_tx, message.clone()).await
                {
                    break;
                }
            }
            Err(err) => {
                dbg!("ws client poll error; err={:?}", err);
                break;
            }
        }
    }

    dbg!("ws client connection closed, good bye");
}

async fn process_message(
    pg: &PgPool,
    claim: &SiClaims,
    outbound_ws_tx: &UnboundedSender<Result<Message, warp::Error>>,
    message: Message,
) -> MessageStream {
    // This `warp::ws::Message` wraps a `tungstenite::protocol::Message` which is an enum, but
    // sadly the warp Message does not expose the underlying enum for pattern mataching. Instead
    // we'll have to iterate through all the variants by hand with if statements
    if message.is_text() {
        dbg!("recv ws text msg, processing; message={:?}", &message);

        match serde_json::from_slice::<ControlOp>(&message.into_bytes()) {
            Ok(control_op) => match control_op {
                ControlOp::Stop(value) => {
                    dbg!("recv control op stop; value={:?}", &value);
                    return MessageStream::Finish;
                }
                ControlOp::LoadData(load_data) => {
                    dbg!("recv control op load data; load_data={:?}", &load_data);
                    let results: Vec<serde_json::Value> = match load_data_model(
                        pg,
                        &load_data.workspace_id,
                        &claim.billing_account_id,
                    )
                    .await
                    {
                        Ok(results) => results,
                        Err(err) => {
                            dbg!("cannot load data: {:?}", &err);
                            return MessageStream::Continue;
                        }
                    };
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
                                Err(err) => {
                                    dbg!("cannot send outbound op 192: {}", err);
                                }
                            },
                            Err(err) => {
                                dbg!("cannot serialize op as json: {}", err);
                            }
                        }
                    }
                    match serde_json::to_string(&UpdateOp::LoadFinished) {
                        Ok(op_json) => match outbound_ws_tx.send(Ok(Message::text(op_json))) {
                            Ok(_) => (),
                            Err(err) => {
                                dbg!("cannot send outbound op 202: {}", err);
                            }
                        },
                        Err(err) => {
                            dbg!("cannot serialize op as json: {}", err);
                        }
                    }
                }
            },
            Err(err) => {
                dbg!("error deserializing control op: {}", err);
            }
        };
    } else if message.is_close() {
        dbg!("recv ws close msg, skipping; message={:?}", message);
    } else if message.is_ping() {
        // Pings are automatically ponged via tungstenite so if we receive this message, there is
        // nothing left to do
        dbg!("recv ws ping msg, skipping; message={:?}", message);
    } else if message.is_pong() {
        dbg!("recv ws pong msg, skipping; message={:?}", message);
    } else if message.is_binary() {
        dbg!(
            "recv ws binary message which is not expected (text only), skipping; message={:#x?}",
            message.as_bytes()
        );
    } else {
        // If we trigger this error, then the underlying `tungstenite::protocol::Message` likely
        // has a new variant that we are not not handling explicitly
        dbg!(
            "recv ws msg of unknown type, likely a new underlying variant, \
            programmer intervention required; message={:?}",
            message
        );
    }

    MessageStream::Continue
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("unauthorized")]
    Unauthorized,
}

pub type UpdateResult<T> = Result<T, UpdateError>;

pub async fn load_data_model(
    _pg: &PgPool,
    workspace_id: impl AsRef<str>,
    billing_account_id: impl AsRef<str>,
) -> UpdateResult<Vec<serde_json::Value>> {
    let _workspace_id = workspace_id.as_ref();
    let _billing_account_id = billing_account_id.as_ref();
    return Ok(Vec::new());
}

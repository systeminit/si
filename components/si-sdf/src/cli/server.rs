use crate::cli::server::command::change_run::{ChangeRun, ChangeRunError};
use crate::data::{Connection, Db};
use crate::models::{
    ApiClaim, ChangeSetError, EditSessionError, EntityError, Event, EventError, EventLog, OpError,
    OutputLine,
};
use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::RwLock;
use tracing::{error, trace, warn};
use warp::ws::{Message, WebSocket};

pub mod command;
pub use crate::cli::server::command::*;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("connection failed")]
    Connection,
    #[error("event error: {0}")]
    Event(#[from] EventError),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("change run error: {0}")]
    ChangeRun(#[from] ChangeRunError),
    #[error("editSession error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("editSession error: {0}")]
    EditSession(#[from] EditSessionError),
    #[error("changeset op error: {0}")]
    Op(#[from] OpError),
}

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ClientCommand {
    NodeChangeRun(NodeChangeRun),
    Stop(serde_json::Value),
}

impl ClientCommand {
    pub async fn into_command(
        self,
        db: &Db,
        billing_account_id: impl AsRef<str>,
    ) -> ServerResult<Command> {
        match self {
            Self::NodeChangeRun(node_change_run) => Ok(Command::ChangeRun(
                node_change_run
                    .into_change_run(db, billing_account_id)
                    .await?,
            )),
            Self::Stop(value) => Ok(Command::Stop(value)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Command {
    ChangeRun(ChangeRun),
    Stop(serde_json::Value),
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum MessageStream {
    Continue,
    Finish,
}

#[derive(Debug, Clone)]
pub struct CommandContext {
    command: Arc<RwLock<Option<Command>>>,
    root_event: Arc<RwLock<Option<Event>>>,
    tracking_ids: Arc<RwLock<Vec<String>>>,
    billing_account_id: Arc<String>,
    api_client_id: Arc<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum CliMessage {
    Event(Event),
    EventLog(EventLog),
    OutputLine(OutputLine),
}

impl CommandContext {
    fn new(
        billing_account_id: impl Into<String>,
        api_client_id: impl Into<String>,
    ) -> CommandContext {
        let billing_account_id = Arc::new(billing_account_id.into());
        let api_client_id = Arc::new(api_client_id.into());

        CommandContext {
            command: Arc::new(RwLock::new(None)),
            root_event: Arc::new(RwLock::new(None)),
            tracking_ids: Arc::new(RwLock::new(Vec::new())),
            billing_account_id,
            api_client_id,
        }
    }

    async fn execute(&self, db: &Db, nats: &Connection) -> ServerResult<()> {
        let command = self.command.read().await;
        match command.as_ref() {
            Some(Command::ChangeRun(ref cr)) => {
                cr.execute(db, nats, self).await?;
            }
            Some(Command::Stop(msg)) => warn!("we shouldn't execute a stop command! bug: {}", msg),
            None => warn!("execute called when no command set; bug!"),
        }
        Ok(())
    }

    async fn set_command(&self, command: Command) {
        let mut wcommand = self.command.write().await;
        wcommand.replace(command);
    }

    async fn set_root_event(&self, event: Event) {
        {
            let mut tracking_ids = self.tracking_ids.write().await;
            tracking_ids.push(event.id.clone());
        }
        {
            let mut root_event = self.root_event.write().await;
            root_event.replace(event);
        }
    }

    async fn add_tracking_id(&self, id: impl Into<String>) {
        let id = id.into();
        let mut tracking_ids = self.tracking_ids.write().await;
        tracking_ids.push(id);
    }
}

pub async fn websocket_run(websocket: WebSocket, db: Db, nats: Connection, claim: ApiClaim) {
    dbg!("doing websocket");
    // Split the socket into a sender and receiver of messages.
    let (ws_tx, mut ws_rx) = websocket.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    //let (outbound_ws_tx, rx): (
    //    mpsc::UnboundedSender<Result<Message, warp::Error>>,
    //    mpsc::UnboundedReceiver<Result<Message, warp::Error>>,
    //) = mpsc::unbounded_channel();
    let (outbound_ws_tx, rx) = mpsc::unbounded_channel();

    let ctx = CommandContext::new(&claim.billing_account_id, &claim.api_client_id);

    // For debugging!
    let claim2 = claim.clone();
    tokio::task::spawn(rx.forward(ws_tx).map(move |result| {
        if let Err(err) = result {
            // Doesn't look like `warp::Error` can deref the inner error, which is rather sad. The
            // "Connection closed normall" 'error' appears to be safe and benign, so we'll ignore
            // this one and warn on all others
            match err.to_string().as_ref() {
                "Connection closed normally" => {
                    trace!("ws cli client send closed normally; err={:?}", err)
                }
                _ => warn!("ws cli client send error; err={}, claim={:?}", err, claim2),
            }
        }
    }));

    let ctx2 = ctx.clone();
    let db2 = db.clone();
    let outbound_ws_tx_from_nats = outbound_ws_tx.clone();
    // Send matching data to the web socket
    match nats
        .subscribe(&format!("{}.>", &claim.billing_account_id))
        .await
    {
        Ok(mut sub) => {
            tokio::task::spawn(async move {
                while let Some(msg) = sub.next().await {
                    match serde_json::from_slice::<serde_json::Value>(&msg.data) {
                        Ok(data_json) => {
                            if let Some(type_name) = data_json["siStorable"]["typeName"].as_str() {
                                if type_name == "event" {
                                    let read_lock = ctx2.root_event.read().await;
                                    match read_lock.as_ref() {
                                        Some(root_event) => {
                                            let event: Event = serde_json::from_value(data_json).expect("bug; we checked type, but cannot deserialize an event");

                                            if event.id == root_event.id {
                                                let data_string = serde_json::to_string(
                                                    &CliMessage::Event(event),
                                                )
                                                .expect("cannot convert event to string, bug!");
                                                match outbound_ws_tx_from_nats
                                                    .send(Ok(Message::text(data_string)))
                                                {
                                                    Ok(_) => (),
                                                    Err(err) => {
                                                        error!(
                                                            "cannot send outbound event; err={:?}; closing",
                                                            err
                                                        );
                                                        return;
                                                    }
                                                }
                                            } else if let Ok(true) =
                                                event.has_parent(&db2, &root_event.id).await
                                            {
                                                let data_string = serde_json::to_string(
                                                    &CliMessage::Event(event),
                                                )
                                                .expect("cannot convert event to string, bug!");
                                                match outbound_ws_tx_from_nats
                                                    .send(Ok(Message::text(data_string)))
                                                {
                                                    Ok(_) => (),
                                                    Err(err) => {
                                                        error!(
                                                            "cannot send outbound event; err={:?}; closing",
                                                            err
                                                        );
                                                        return;
                                                    }
                                                }
                                            }
                                        }
                                        None => {}
                                    }
                                } else if type_name == "eventLog" {
                                    let read_lock = ctx2.root_event.read().await;
                                    match read_lock.as_ref() {
                                        Some(root_event) => {
                                            let event_log: EventLog = serde_json::from_value(data_json).expect("bug; we checked type, but cannot deserialize an eventLog");

                                            if let Ok(true) =
                                                event_log.has_parent(&db2, &root_event.id).await
                                            {
                                                let data_string = serde_json::to_string(
                                                    &CliMessage::EventLog(event_log),
                                                )
                                                .expect("cannot convert eventLog to string, bug!");
                                                match outbound_ws_tx_from_nats
                                                    .send(Ok(Message::text(data_string)))
                                                {
                                                    Ok(_) => (),
                                                    Err(err) => {
                                                        error!(
                                                            "cannot send outbound event; err={:?}; closing",
                                                            err
                                                        );
                                                        return;
                                                    }
                                                }
                                            }
                                        }
                                        None => {}
                                    }
                                } else if type_name == "outputLine" {
                                    let read_lock = ctx2.root_event.read().await;
                                    match read_lock.as_ref() {
                                        Some(root_event) => {
                                            let output_line: OutputLine = serde_json::from_value(data_json).expect("bug; we checked type, but cannot deserialize an outputLine");

                                            if let Ok(true) =
                                                output_line.has_parent(&db2, &root_event.id).await
                                            {
                                                let data_string = serde_json::to_string(
                                                    &CliMessage::OutputLine(output_line),
                                                )
                                                .expect(
                                                    "cannot convert outputLine to string, bug!",
                                                );
                                                match outbound_ws_tx_from_nats
                                                    .send(Ok(Message::text(data_string)))
                                                {
                                                    Ok(_) => (),
                                                    Err(err) => {
                                                        error!(
                                                            "cannot send outbound event; err={:?}; closing",
                                                            err
                                                        );
                                                        return;
                                                    }
                                                }
                                            }
                                        }
                                        None => {}
                                    }
                                }
                            }
                        }
                        Err(err) => error!("bad data from nats: {} / {:#?}", err, msg),
                    }
                }
            });
        }
        Err(err) => error!("websocket error creating subscriber: {}", err),
    }

    // Listen to Commands from the websocket
    while let Some(message_result) = ws_rx.next().await {
        match message_result {
            Ok(message) => {
                match process_message(&db, &nats, &claim, &outbound_ws_tx, &ctx, message.clone())
                    .await
                {
                    MessageStream::Finish => break,
                    MessageStream::Continue => {}
                }
            }
            Err(err) => {
                trace!("ws cli client poll error; err={:?}", err);
                break;
            }
        }
    }
    match outbound_ws_tx.send(Ok(Message::text("stop"))) {
        Ok(_) => (),
        Err(err) => {
            error!("cannot send outbound stop event; err={:?}; closing", err);
        }
    }

    outbound_ws_tx
        .send(Ok(Message::close_with(1000 as u16, "work complete")))
        .expect("cannot close");

    trace!("ws cli client connection closed, good bye");
}

async fn process_message(
    db: &Db,
    nats: &Connection,
    claim: &ApiClaim,
    outbound_ws_tx: &UnboundedSender<Result<Message, warp::Error>>,
    ctx: &CommandContext,
    message: Message,
) -> MessageStream {
    // This `warp::ws::Message` wraps a `tungstenite::protocol::Message` which is an enum, but
    // sadly the warp Message does not expose the underlying enum for pattern mataching. Instead
    // we'll have to iterate through all the variants by hand with if statements
    if message.is_text() {
        trace!("recv ws text msg, processing; message={:?}", message);

        dbg!(&message);
        // Deserialize the `ClientCommand`
        let client_command = match serde_json::from_slice::<ClientCommand>(&message.into_bytes()) {
            Ok(client_command) => client_command,
            Err(err) => {
                tracing::warn!("error deserializing client command: {}", err);
                let _e = outbound_ws_tx.send(Ok(Message::close_with(
                    4001 as u16,
                    format!("error deserializing client command: {}", err),
                )));
                return MessageStream::Finish;
            }
        };
        match client_command
            .into_command(db, &claim.billing_account_id)
            .await
        {
            Ok(Command::Stop(_)) => {
                return MessageStream::Finish;
            }
            Ok(command @ Command::ChangeRun(_)) => {
                ctx.set_command(command.clone()).await;
                let db2 = db.clone();
                let nats2 = nats.clone();
                let ctx2 = ctx.clone();
                let outbound_ws_tx2 = outbound_ws_tx.clone();
                tokio::task::spawn(async move {
                    dbg!("running change run command execute function");
                    match ctx2.execute(&db2, &nats2).await {
                        Ok(()) => {
                            let _e = outbound_ws_tx2
                                .send(Ok(Message::close_with(1000 as u16, "closed")));
                        }
                        Err(e) => {
                            let _e = outbound_ws_tx2.send(Ok(Message::close_with(
                                4001 as u16,
                                format!("error: {}", e),
                            )));
                        }
                    }
                });
            }
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

use axum::{
    extract::{ws::WebSocket, Extension, WebSocketUpgrade},
    response::IntoResponse,
};
use dal::BillingAccountId;
use si_data::NatsClient;
use telemetry::prelude::*;
use tokio::sync::broadcast;

use crate::server::{
    extract::{Authorization, Nats, WsAuthorization},
    routes::ShutdownBroadcast,
};

#[instrument(skip(wsu, nats))]
#[allow(clippy::unused_async)]
pub async fn billing_account_updates(
    wsu: WebSocketUpgrade,
    Nats(nats): Nats,
    WsAuthorization(claim): WsAuthorization,
    Extension(shutdown_broadcast): Extension<ShutdownBroadcast>,
) -> impl IntoResponse {
    async fn handle_socket(
        socket: WebSocket,
        nats: NatsClient,
        mut shutdown: broadcast::Receiver<()>,
        billing_account_id: BillingAccountId,
    ) {
        tokio::select! {
            _ = run_billing_account_updates_proto(socket, nats, billing_account_id) => {
                trace!("finished billing_account_updates proto");
            }
            _ = shutdown.recv() => {
                trace!("billing_account_updates received shutdown, ending session");
            }
            else => {
                trace!("returning from billing_account_updates, all select arms closed");
            }
        }
    }

    let shutdown = shutdown_broadcast.subscribe();
    wsu.on_upgrade(move |socket| handle_socket(socket, nats, shutdown, claim.billing_account_id))
}

async fn run_billing_account_updates_proto(
    mut socket: WebSocket,
    nats: NatsClient,
    billing_account_id: BillingAccountId,
) {
    let proto = match billing_account_updates::run(nats, billing_account_id)
        .start()
        .await
    {
        Ok(started) => started,
        Err(err) => {
            // This is likely due to nats failing to subscribe to the required topic, which is
            // suspicious
            warn!(error = ?err, "protocol failed to start");
            return;
        }
    };
    let proto = match proto.process(&mut socket).await {
        Ok(processed) => processed,
        Err(err) => {
            // An error is most likely returned when the client side terminates the websocket
            // session or if a network partition occurs, so this is our "normal" behavior
            trace!(error = ?err, "failed to cleanly complete update stream");
            return;
        }
    };
    if let Err(err) = proto.finish(socket).await {
        // We'd like finish to complete cleanly
        warn!(error = ?err, "failed to finish protocol");
    }
}

mod billing_account_updates {
    use std::error::Error;

    use axum::extract::ws::{self, WebSocket};
    use dal::BillingAccountId;
    use futures::{StreamExt, TryStreamExt};
    use si_data::{nats::Subscription, NatsClient, NatsError};
    use telemetry::prelude::*;
    use thiserror::Error;
    use tokio_tungstenite::tungstenite;

    pub fn run(nats: NatsClient, billing_account_id: BillingAccountId) -> BillingAccountUpdates {
        BillingAccountUpdates {
            nats,
            billing_account_id,
        }
    }

    #[derive(Debug, Error)]
    pub enum BillingAccountUpdatesError {
        #[error("error processing nats message from subscription")]
        NatsIo(#[source] NatsError),
        #[error("failed to subscribe to subject {1}")]
        Subscribe(#[source] NatsError, String),
        #[error("error when closing websocket")]
        WsClose(#[source] axum::Error),
        #[error("error when sending websocket message")]
        WsSendIo(#[source] axum::Error),
    }

    type Result<T> = std::result::Result<T, BillingAccountUpdatesError>;

    #[derive(Debug)]
    pub struct BillingAccountUpdates {
        nats: NatsClient,
        billing_account_id: BillingAccountId,
    }

    impl BillingAccountUpdates {
        pub async fn start(self) -> Result<BillingAccountUpdatesStarted> {
            let subject = format!("si.billing_account_id.{}.>", self.billing_account_id.to_string());
            let subscription = self
                .nats
                .subscribe(&subject)
                .await
                .map_err(|err| BillingAccountUpdatesError::Subscribe(err, subject))?;

            Ok(BillingAccountUpdatesStarted { subscription })
        }
    }

    #[derive(Debug)]
    pub struct BillingAccountUpdatesStarted {
        subscription: Subscription,
    }

    impl BillingAccountUpdatesStarted {
        pub async fn process(self, ws: &mut WebSocket) -> Result<BillingAccountUpdatesClosing> {
            // Send all messages down the WebSocket until and unless an error is encountered, the
            // client websocket connection is closed, or the nats subscription naturally closes
            while let Some(nats_msg) = self
                .subscription
                .async_next()
                .await
                .map_err(BillingAccountUpdatesError::NatsIo)?
            {
                let msg = ws::Message::Text(String::from_utf8_lossy(nats_msg.data()).to_string());

                if let Err(err) = ws.send(msg).await {
                    match err
                        .source()
                        .map(|err| err.downcast_ref::<tungstenite::Error>())
                        .flatten()
                    {
                        Some(ws_err) => match ws_err {
                            // If the websocket has cleanly closed, we should cleanly finish as
                            // well--this is not an error condition
                            tungstenite::Error::ConnectionClosed
                            | tungstenite::Error::AlreadyClosed => {
                                trace!("websocket has cleanly closed, ending");
                                return Ok(BillingAccountUpdatesClosing { ws_is_closed: true });
                            }
                            _ => return Err(BillingAccountUpdatesError::WsSendIo(err)),
                        },
                        None => return Err(BillingAccountUpdatesError::WsSendIo(err)),
                    }
                }
            }

            Ok(BillingAccountUpdatesClosing {
                ws_is_closed: false,
            })
        }
    }

    #[derive(Debug)]
    pub struct BillingAccountUpdatesClosing {
        ws_is_closed: bool,
    }

    impl BillingAccountUpdatesClosing {
        pub async fn finish(self, ws: WebSocket) -> Result<()> {
            if !self.ws_is_closed {
                ws.close()
                    .await
                    .map_err(BillingAccountUpdatesError::WsClose)?;
            }
            Ok(())
        }
    }
}

use std::time::Duration;

use axum::extract::ws::WebSocket;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{sync::mpsc, time};

use crate::{ShutdownSource, WebSocketMessage};

pub fn run(keepalive_tx: mpsc::Sender<()>, timeout: Duration) -> WatchRun {
    WatchRun {
        keepalive_tx,
        timeout,
    }
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WatchError {
    #[error("failed to send server keepalive")]
    Mpsc(#[from] mpsc::error::SendError<()>),
    #[error("failed to send websocket message")]
    WSSendIO(#[from] axum::Error),
}

type Result<T> = std::result::Result<T, WatchError>;

#[derive(Debug)]
pub struct WatchRun {
    keepalive_tx: mpsc::Sender<()>,
    timeout: Duration,
}

impl WatchRun {
    pub async fn start(self, ws: &mut WebSocket) -> Result<()> {
        let mut heartbeat_interval = time::interval(
            self.timeout
                .checked_div(3)
                .expect("only fails when arg is 0"),
        );
        let msg = vec![];

        loop {
            let _instant = heartbeat_interval.tick().await;

            trace!("sending server keepalive");
            self.keepalive_tx.send(()).await?;
            trace!("sending websocket ping");
            ws.send(WebSocketMessage::Ping(msg.clone())).await?;
        }
    }
}

pub async fn watch_timeout_task(
    watch_timeout: Duration,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
    mut keepalive_rx: mpsc::Receiver<()>,
) {
    loop {
        match time::timeout(watch_timeout, keepalive_rx.recv()).await {
            Ok(Some(_)) => {
                // Got a keepalive
                trace!("watch_timeout_task got keepalive");
            }
            Ok(None) | Err(_) => {
                // Timeout has elapsed
                info!("watch_timeout_task timeout elapsed");
                if shutdown_tx
                    .send(ShutdownSource::WatchTimeout)
                    .await
                    .is_err()
                {
                    warn!("failed to send shutdown, receiver has already dropped");
                }
                break;
            }
        }
    }
}

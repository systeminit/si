use std::sync::Arc;

use tokio::runtime::Runtime;
use telemetry::prelude::*;

use crate::{error::SiFullStackResult, Config, SiFullStackError};

#[derive(Debug)]
pub struct Client {
    config: Config,
    join_handle: std::thread::JoinHandle<SiFullStackResult<()>>,
    shutdown_tx: tokio::sync::oneshot::Sender<bool>,
}

impl Client {
    pub fn new(
        config: Config,
        join_handle: std::thread::JoinHandle<SiFullStackResult<()>>,
        shutdown_tx: tokio::sync::oneshot::Sender<bool>,
    ) -> Self {
        Client {
            config,
            join_handle,
            shutdown_tx,
        }
    }

    pub fn shutdown(self) -> SiFullStackResult<()> {
        if let Err(_e) = self.shutdown_tx.send(true) {
            debug!("Full Stack Server has already been destroyed; cannot send shutdown signal, obviously");
        }
        match self.join_handle.join() {
            Ok(Ok(())) => debug!("Graceful shutdown of full stack server complete"),
            // But maybe not happening during shutdown, since the thread could've died
            // otherwise. :)
            Ok(Err(err)) => {
                error!(?err, "Full stack server error, discovered on shutdown");
                return Err(err);
            },
            Err(err) => {
                error!(?err, "Full stack server panic-ed");
                return Err(SiFullStackError::ServerPanic(format!("{:?}", err)));
            }
        }
        Ok(())
    }
}

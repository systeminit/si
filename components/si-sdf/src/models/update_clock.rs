use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use crate::data::PgPool;

use tracing::error;

use si_settings::Settings;
use std::sync::{Arc, Mutex};

// Update Clocks have an epoch and an update_count. The update count updates until
// it hits 9007199254740992 - 1, just below the max size javascript can serialize
// as an integer. Then it increments the epoch by 1. If we run out of epoch (which
// is also the same max size), then you get a panic from the database, and the
// workspace will grind to a halt. You'll need to figure out what to do - maybe
// implement an age? A saga? What's bigger than an epoch?
//
// You might also ask yourself - why isn't this just a call? Good question! the reason
// is that we use transactions in the request handlers. Since update clocks are tied
// to the workspace, it means any update in the workspace would need to lock this row to
// ensure it has a unique value when it is finished, which means the whole thing will be
// "racy as fuck".
//
// Hence, it has a seperate handle to the entire database pool, makes its own connection,
// and its own seperate transaction. Then you can safely use that number inside your
// (perhaps longer running) real transaction safely.
//
// We didn't use a sequence because we don't want a table for every workspace.
//
// It runs in its own reactor, so that it can be used across the multi-threaded
// test suite, and to make sure that it won't ever wind up blocking the main
// runtime.

static mut UPDATE_CLOCK_CLIENT: Option<UpdateClockClient> = None;
lazy_static! {
    pub static ref UPDATE_CLIENT_LOCK: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

pub async fn next_update_clock(clock_id: impl Into<String>) -> UpdateClockResult<UpdateClock> {
    unsafe {
        UPDATE_CLOCK_CLIENT
            .as_mut()
            .cloned()
            .expect("hey homie, you really need to initialize the client before use")
            .update(clock_id)
            .await
    }
}

pub fn init_update_clock_service(s: &Settings) {
    let mut finished = UPDATE_CLIENT_LOCK
        .lock()
        .expect("cannot get an update client lock");

    if *finished {
        return;
    }
    let settings = s.clone();
    let (uc_service, uc_client) = crate::models::UpdateClockService::new();
    unsafe {
        UPDATE_CLOCK_CLIENT = Some(uc_client);
    }

    std::thread::spawn(move || {
        let mut rt = tokio::runtime::Builder::new()
            .enable_all()
            .threaded_scheduler()
            .core_threads(5)
            .build()
            .expect("cannot start update clock service runtime");
        rt.block_on(async move {
            let pg = PgPool::new(&settings.pg)
                .await
                .expect("cannot connect to postgres");
            uc_service.run(pg).await
        });
    });

    *finished = true;
}

#[derive(Error, Debug)]
pub enum UpdateClockError {
    #[error("update count for this clock exceeded; something is wrong!")]
    UpdateCountExceeded,
    #[error("pg pool error")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("oneshot response error: {0}")]
    OneshotRecv(#[from] oneshot::error::RecvError),
    #[error("server send error: {0}")]
    Mpsc(#[from] mpsc::error::SendError<(String, oneshot::Sender<UpdateClock>)>),
}

pub type UpdateClockResult<T> = Result<T, UpdateClockError>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClock {
    pub epoch: i64, // These are limited in the database to javascript size
    pub update_count: i64,
}

#[derive(Debug, Clone)]
pub struct UpdateClockClient {
    tx: mpsc::UnboundedSender<(String, oneshot::Sender<UpdateClock>)>,
}

impl UpdateClockClient {
    pub fn new(tx: mpsc::UnboundedSender<(String, oneshot::Sender<UpdateClock>)>) -> Self {
        UpdateClockClient { tx }
    }

    pub async fn update(&mut self, clock_id: impl Into<String>) -> UpdateClockResult<UpdateClock> {
        let clock_id = clock_id.into();
        let (req_tx, req_rx) = oneshot::channel();
        self.tx.send((clock_id, req_tx))?;
        let update_clock = req_rx.await?;
        Ok(update_clock)
    }
}

#[derive(Debug)]
pub struct UpdateClockService {
    rx: mpsc::UnboundedReceiver<(String, oneshot::Sender<UpdateClock>)>,
}

impl UpdateClockService {
    pub fn new() -> (UpdateClockService, UpdateClockClient) {
        let (tx, rx) = mpsc::unbounded_channel();
        let service = UpdateClockService { rx };
        let client = UpdateClockClient { tx };
        (service, client)
    }

    pub async fn run(mut self, pg: PgPool) {
        while let Some((clock_id, response_channel)) = self.rx.recv().await {
            let pool = pg.clone();
            tokio::spawn(async move {
                match UpdateClockService::process(pool, clock_id, response_channel).await {
                    Ok(_) => {}
                    Err(e) => error!(?e, "update clock service run processing failed"),
                }
            });
        }
    }

    pub async fn process(
        pg: PgPool,
        clock_id: String,
        response_channel: oneshot::Sender<UpdateClock>,
    ) -> UpdateClockResult<()> {
        let conn = pg.pool.get().await?;
        let row = conn
            .query_one(
                "SELECT new_epoch, new_update_count FROM update_clock_v1($1)",
                &[&clock_id],
            )
            .await?;
        let new_epoch: i64 = row.try_get("new_epoch")?;
        let new_update_count: i64 = row.try_get("new_update_count")?;
        let update_clock = UpdateClock {
            epoch: new_epoch,
            update_count: new_update_count,
        };
        match response_channel.send(update_clock) {
            Ok(_) => {}
            Err(clock) => tracing::debug!(
                ?clock,
                ?clock_id,
                "could not send clock back to requestor; possible bad mojo?"
            ),
        }
        Ok(())
    }
}

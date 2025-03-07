use std::sync::Arc;

use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::sync::{Mutex, MutexGuard};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum NatsClientError {
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
}

#[derive(Clone, Debug)]
pub struct NatsClient {
    config: si_data_nats::NatsConfig,
    inner: Arc<Mutex<si_data_nats::NatsClient>>,
}

type Result<T> = std::result::Result<T, NatsClientError>;

impl NatsClient {
    pub async fn new(config: si_data_nats::NatsConfig) -> Result<Self> {
        metric!(counter.veritech.nats_client.try_lock_attempt = 0);
        metric!(counter.veritech.nats_client.hot_swap = 0);

        let raw = si_data_nats::NatsClient::new_for_veritech(&config).await?;

        Ok(Self {
            config,
            inner: Arc::new(Mutex::new(raw)),
        })
    }

    pub async fn jetstream_context_no_hot_swap(&self) -> si_data_nats::jetstream::Context {
        let guard = self.lock().await;
        let cloned_client = guard.clone();
        let jetstream_context = si_data_nats::jetstream::new(cloned_client);
        drop(guard);
        jetstream_context
    }

    pub async fn lock(&self) -> MutexGuard<'_, si_data_nats::NatsClient> {
        loop {
            if let Ok(guard) = self.inner.try_lock() {
                return guard;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            metric!(counter.veritech.nats_client.try_lock_attempt = 1);
        }
    }

    pub async fn lock_hot_swap_client_unlock(&self) -> Result<()> {
        let mut inner = self.lock().await;
        *inner = si_data_nats::NatsClient::new_for_veritech(&self.config).await?;
        drop(inner);
        metric!(counter.veritech.nats_client.hot_swap = 1);
        Ok(())
    }
}

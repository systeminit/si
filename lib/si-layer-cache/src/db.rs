use si_data_nats::NatsClient;
use si_data_pg::PgPool;
use std::path::Path;

use crate::{
    error::LayerDbResult,
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterServer},
};
use tokio::sync::mpsc;

use self::cas::CasDb;

pub mod cas;

#[derive(Debug, Clone)]
pub struct LayerDb {
    cas: CasDb,
    sled: sled::Db,
    pg_pool: PgPool,
    nats_client: NatsClient,
    persister_client: PersisterClient,
}

impl LayerDb {
    pub async fn new(
        disk_path: impl AsRef<Path>,
        pg_pool: PgPool,
        nats_client: NatsClient,
    ) -> LayerDbResult<Self> {
        let disk_path = disk_path.as_ref();
        let sled = sled::open(disk_path)?;

        let (tx, rx) = mpsc::unbounded_channel();
        let persister_client = PersisterClient::new(tx);
        let persister_server_sled = sled.clone();
        let persister_server_pg_pool = pg_pool.clone();
        let persister_server_nats_client = nats_client.clone();
        tokio::spawn(async move {
            PersisterServer::start(
                rx,
                persister_server_sled,
                persister_server_pg_pool,
                persister_server_nats_client,
            )
            .await
        });

        let cas_cache = LayerCache::new("cas", sled.clone(), pg_pool.clone()).await?;
        let cas = CasDb::new(cas_cache, persister_client.clone());

        Ok(LayerDb {
            cas,
            sled,
            pg_pool,
            persister_client,
            nats_client,
        })
    }

    pub fn sled(&self) -> &sled::Db {
        &self.sled
    }

    pub fn pg_pool(&self) -> &PgPool {
        &self.pg_pool
    }

    pub fn nats_client(&self) -> &NatsClient {
        &self.nats_client
    }

    pub fn persister_client(&self) -> &PersisterClient {
        &self.persister_client
    }

    pub fn cas(&self) -> &CasDb {
        &self.cas
    }
}

use serde::de::DeserializeOwned;
use serde::Serialize;
use si_data_pg::PgPool;
use std::collections::HashMap;
use std::time::Instant;
use telemetry::prelude::*;

use crate::hash::ContentHash;
use crate::pair::ContentPair;
use crate::store::{Store, StoreResult};
use crate::PgStoreTools;

pub(crate) mod tools;

/// A content store backed by Postgres.
#[derive(Debug, Clone)]
pub struct PgStore {
    inner: HashMap<ContentHash, PgStoreItem>,
    pg_pool: PgPool,
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
struct PgStoreItem {
    value: serde_json::Value,
    written: bool,
}

impl PgStoreItem {
    fn new(value: serde_json::Value) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}

impl PgStore {
    /// Create a new [`PgStore`] from a given [`PgPool`].
    pub async fn new(pg_pool: PgPool) -> StoreResult<Self> {
        Ok(Self {
            inner: Default::default(),
            pg_pool,
        })
    }

    /// Create a new [`PgStore`] from a given [`PgPool`].
    pub async fn new_production() -> StoreResult<Self> {
        let pg_pool = PgStoreTools::new_production_pg_pool().await?;
        Ok(Self {
            inner: Default::default(),
            pg_pool,
        })
    }

    /// Migrate the content store database
    pub async fn migrate() -> StoreResult<()> {
        let pg_pool = PgStoreTools::new_production_pg_pool().await?;
        PgStoreTools::migrate(&pg_pool).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Store for PgStore {
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn add<T>(&mut self, object: &T) -> StoreResult<ContentHash>
    where
        T: Serialize + ?Sized,
    {
        let value = serde_json::to_value(object)?;
        let key = ContentHash::new(value.to_string().as_bytes());
        self.inner.insert(key, PgStoreItem::new(value));
        Ok(key)
    }

    async fn get<T>(&mut self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let object = match self.inner.get(key) {
            Some(item) => serde_json::from_value(item.value.to_owned())?,
            None => match ContentPair::find(&self.pg_pool, key).await? {
                Some(content_pair) => {
                    let encoded = content_pair.value();
                    let decoded = serde_json::from_value(encoded.to_owned())?;
                    self.add(encoded)?;

                    decoded
                }
                None => return Ok(None),
            },
        };
        Ok(Some(object))
    }

    async fn get_bulk<T>(&mut self, keys: &[ContentHash]) -> StoreResult<HashMap<ContentHash, T>>
    where
        T: DeserializeOwned + std::marker::Send,
    {
        let get_bulk_start = Instant::now();
        let mut result = HashMap::new();
        let mut keys_to_fetch = vec![];

        for key in keys {
            match self.inner.get(key) {
                Some(item) => {
                    result.insert(*key, serde_json::from_value(item.value.to_owned())?);
                }
                None => keys_to_fetch.push(*key),
            }
        }

        for pair in ContentPair::find_many(&self.pg_pool, keys_to_fetch.as_slice()).await? {
            let encoded = pair.value();
            result.insert(pair.key()?, serde_json::from_value(encoded.to_owned())?);
            self.add(encoded)?;
        }

        info!("get_bulk: {:?}", get_bulk_start.elapsed());

        Ok(result)
    }

    async fn write(&mut self) -> StoreResult<()> {
        for (key, item) in self.inner.iter_mut() {
            if !item.written {
                ContentPair::find_or_create(&self.pg_pool, key.to_owned(), item.value.clone())
                    .await?;
                item.written = true;
            }
        }
        Ok(())
    }
}

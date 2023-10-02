use serde::de::DeserializeOwned;
use serde::Serialize;
use si_data_pg::PgPool;
use std::collections::HashMap;

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
    value: Vec<u8>,
    written: bool,
}

impl PgStoreItem {
    fn new(value: Vec<u8>) -> Self {
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
        let value = serde_json::to_vec(object)?;
        let key = ContentHash::new(&value);
        self.inner.insert(key, PgStoreItem::new(value));
        Ok(key)
    }

    async fn get<T>(&mut self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let object = match self.inner.get(key) {
            Some(item) => serde_json::from_slice(&item.value)?,
            None => match ContentPair::find(&self.pg_pool, key).await? {
                Some(content_pair) => {
                    let bytes = content_pair.value();
                    self.add(bytes)?;
                    serde_json::from_slice(bytes)?
                }
                None => return Ok(None),
            },
        };
        Ok(Some(object))
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

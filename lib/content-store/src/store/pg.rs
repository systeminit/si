use serde::de::DeserializeOwned;
use serde::Serialize;
use si_data_pg::PgPool;
use std::collections::HashMap;

use crate::pair::ContentPair;
use crate::store::{Store, StoreResult};
use crate::ContentHash;
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

    /// Migrate the content store database
    pub async fn migrate(pg_pool: &PgPool) -> StoreResult<()> {
        PgStoreTools::migrate(pg_pool).await?;

        Ok(())
    }

    /// Access the internal pg_pool
    pub fn pg_pool(&self) -> &PgPool {
        &self.pg_pool
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
        let value = postcard::to_stdvec(object)?;
        let key = ContentHash::new(value.as_slice());
        self.inner.insert(key, PgStoreItem::new(value));
        Ok(key)
    }

    async fn get<T>(&mut self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let object = match self.inner.get(key) {
            Some(item) => postcard::from_bytes(&item.value)?,
            None => match ContentPair::find(&self.pg_pool, key).await? {
                Some(content_pair) => {
                    let encoded = content_pair.value();
                    let decoded = postcard::from_bytes(encoded)?;
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
        let mut result = HashMap::new();
        let mut keys_to_fetch = vec![];

        for key in keys {
            match self.inner.get(key) {
                Some(item) => {
                    result.insert(*key, postcard::from_bytes(&item.value)?);
                }
                None => keys_to_fetch.push(*key),
            }
        }

        for pair in ContentPair::find_many(&self.pg_pool, keys_to_fetch.as_slice()).await? {
            let encoded = pair.value();
            result.insert(pair.key()?, postcard::from_bytes(encoded)?);
            self.add(encoded)?;
        }
        Ok(result)
    }

    async fn write(&mut self) -> StoreResult<()> {
        for (key, item) in self.inner.iter_mut() {
            if !item.written {
                ContentPair::new(&self.pg_pool, key.to_owned(), &item.value).await?;
                item.written = true;
            }
        }
        Ok(())
    }
}

use crate::hash::ContentHash;
use crate::pair::ContentPair;
use crate::store::{Store, StoreItem, StoreResult};
use serde::de::DeserializeOwned;
use serde::Serialize;
use si_data_pg::PgPool;
use std::collections::HashMap;

mod migrate;

#[derive(Debug)]
pub struct PgStore {
    inner: HashMap<ContentHash, StoreItem>,
    pg_pool: PgPool,
}

impl PgStore {
    pub async fn new(pg_pool: PgPool) -> StoreResult<Self> {
        Ok(Self {
            inner: Default::default(),
            pg_pool,
        })
    }
}

#[async_trait::async_trait]
impl Store for PgStore {
    fn is_empty(&self) -> bool {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    async fn get<T>(&self, _key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        todo!()
    }

    // NOTE(nick): existing entries must remain immutable.
    fn add<T>(&mut self, value: T) -> StoreResult<(ContentHash, bool)>
    where
        T: Serialize + ToOwned,
    {
        let value = serde_json::to_value(value)?;
        let hash = ContentHash::from(&value);
        let already_in_store = self.inner.contains_key(&hash);
        if !already_in_store {
            // NOTE(nick): we DO NOT check that it is in the database because it does not matter.
            // We wait until write time to talk to the database.
            self.inner.insert(
                hash,
                StoreItem {
                    value,
                    written: false,
                },
            );
        }
        Ok((hash, already_in_store))
    }

    // TODO(nick): actually do stuff with the database.
    async fn write(&mut self) -> StoreResult<()> {
        for (key, value) in self.inner.iter_mut() {
            if !value.written {
                // TODO(nick): perform find or create in the database. Either way, we need to
                // set "processed" to true for the next time we perform a batch write.
                ContentPair::find_or_create(&self.pg_pool, key.to_owned(), value.value.clone())
                    .await?;
                value.written = true;
            }
        }
        Ok(())
    }
}

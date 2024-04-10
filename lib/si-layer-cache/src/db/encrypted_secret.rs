use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};

use serde::{de::DeserializeOwned, Serialize};
use si_events::{Actor, EncryptedSecretKey, Tenancy, WebEvent};

use crate::{
    error::LayerDbResult,
    event::{LayeredEvent, LayeredEventKind},
    layer_cache::LayerCache,
    persister::{PersisterClient, PersisterStatusReader},
    LayerDbError,
};

use super::serialize;

const KEYWORD_SINGULAR: &str = "encrypted_secret";
const KEYWORD_PLURAL: &str = "encrypted_secrets";

pub const PARTITION_KEY: &str = KEYWORD_PLURAL;
pub const DBNAME: &str = KEYWORD_PLURAL;
pub const CACHE_NAME: &str = KEYWORD_PLURAL;
pub const SORT_KEY: &str = KEYWORD_SINGULAR;

#[derive(Debug, Clone)]
pub struct EncryptedSecretDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub cache: LayerCache<Arc<V>>,
    persister_client: PersisterClient,
}

impl<V> EncryptedSecretDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(cache: LayerCache<Arc<V>>, persister_client: PersisterClient) -> Self {
        EncryptedSecretDb {
            cache,
            persister_client,
        }
    }

    pub async fn write(
        &self,
        key: EncryptedSecretKey,
        value: Arc<V>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<PersisterStatusReader> {
        let postcard_value = serialize::to_vec(&value)?;

        let cache_key: Arc<str> = key.to_string().into();

        self.cache.insert(cache_key.clone(), value.clone()).await;

        let event = LayeredEvent::new(
            LayeredEventKind::EncryptedSecretInsertion,
            Arc::new(DBNAME.to_string()),
            cache_key,
            Arc::new(postcard_value),
            Arc::new(SORT_KEY.to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;

        Ok(reader)
    }

    pub async fn read(&self, key: &EncryptedSecretKey) -> LayerDbResult<Option<Arc<V>>> {
        self.cache.get(key.to_string().into()).await
    }

    /// We often need to extract the value from the arc by cloning it (although
    /// this should be avoided for large values). This will do that, and also
    /// helpfully convert the value to the type we want to deal with
    pub async fn try_read_as<T>(&self, key: &EncryptedSecretKey) -> LayerDbResult<Option<T>>
    where
        V: TryInto<T>,
        <V as TryInto<T>>::Error: Display,
    {
        Ok(match self.read(key).await? {
            None => None,
            Some(arc_v) => Some(
                arc_v
                    .as_ref()
                    .clone()
                    .try_into()
                    .map_err(|err| LayerDbError::ContentConversion(err.to_string()))?,
            ),
        })
    }

    pub async fn read_many(
        &self,
        keys: &[EncryptedSecretKey],
    ) -> LayerDbResult<HashMap<EncryptedSecretKey, Arc<V>>> {
        self.cache.get_bulk(keys).await
    }

    pub async fn try_read_many_as<T>(
        &self,
        keys: &[EncryptedSecretKey],
    ) -> LayerDbResult<HashMap<EncryptedSecretKey, T>>
    where
        V: TryInto<T>,
        <V as TryInto<T>>::Error: Display,
    {
        let mut result = HashMap::new();
        for (key, arc_v) in self.cache.get_bulk(keys).await? {
            result.insert(
                key,
                arc_v
                    .as_ref()
                    .clone()
                    .try_into()
                    .map_err(|err| LayerDbError::ContentConversion(err.to_string()))?,
            );
        }

        Ok(result)
    }
}

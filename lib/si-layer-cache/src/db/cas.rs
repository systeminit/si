use std::{
    collections::HashMap,
    fmt::Display,
    sync::Arc,
};

use serde::{
    Serialize,
    de::DeserializeOwned,
};
use si_events::{
    Actor,
    ContentHash,
    Tenancy,
    WebEvent,
};
use telemetry::prelude::*;

use super::serialize;
use crate::{
    LayerDbError,
    error::LayerDbResult,
    event::{
        LayeredEvent,
        LayeredEventKind,
    },
    layer_cache::LayerCache,
    persister::{
        PersisterClient,
        PersisterStatusReader,
    },
};

pub const DBNAME: &str = "cas";
pub const CACHE_NAME: &str = "cas";
pub const PARTITION_KEY: &str = "cas";

#[derive(Debug, Clone)]
pub struct CasDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub cache: Arc<LayerCache<Arc<V>>>,
    persister_client: PersisterClient,
}

impl<V> CasDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(cache: Arc<LayerCache<Arc<V>>>, persister_client: PersisterClient) -> Self {
        CasDb {
            cache,
            persister_client,
        }
    }

    #[instrument(name = "cas.write", level = "debug", skip_all)]
    pub fn write(
        &self,
        value: Arc<V>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<(ContentHash, PersisterStatusReader)> {
        let (postcard_value, size_hint) = serialize::to_vec(&value)?;
        let key = ContentHash::new(&postcard_value);
        let cache_key: Arc<str> = key.to_string().into();

        self.cache
            .insert(cache_key.clone(), value.clone(), size_hint);

        let event = LayeredEvent::new(
            LayeredEventKind::CasInsertion,
            Arc::new(DBNAME.to_string()),
            cache_key,
            Arc::new(postcard_value),
            Arc::new("cas".to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;

        Ok((key, reader))
    }

    pub async fn read(&self, key: &ContentHash) -> LayerDbResult<Option<Arc<V>>> {
        self.cache.get(key.to_string().into()).await
    }

    /// We often need to extract the value from the arc by cloning it (although
    /// this should be avoided for large values). This will do that, and also
    /// helpfully convert the value to the type we want to deal with
    pub async fn try_read_as<T>(&self, key: &ContentHash) -> LayerDbResult<Option<T>>
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
    #[instrument(name = "cas.read_many", level = "debug", skip_all)]
    pub async fn read_many(
        &self,
        keys: &[ContentHash],
    ) -> LayerDbResult<HashMap<ContentHash, Arc<V>>> {
        self.cache.get_bulk(keys).await
    }

    #[instrument(name = "cas.try_read_many_as", level = "debug", skip_all)]
    pub async fn try_read_many_as<T>(
        &self,
        keys: &[ContentHash],
    ) -> LayerDbResult<HashMap<ContentHash, T>>
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

    #[instrument(name = "cas.write_bytes_to_durable_storage", level = "debug", skip_all)]
    pub async fn write_bytes_to_durable_storage(
        &self,
        key: &ContentHash,
        bytes: &[u8],
    ) -> LayerDbResult<()> {
        let key = key.to_string();
        self.cache.pg().insert(&key, "cas", bytes).await?;

        Ok(())
    }
}

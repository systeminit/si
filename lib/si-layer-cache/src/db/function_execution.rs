use std::string::ParseError;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Display};

use serde::{de::DeserializeOwned, Serialize};
use si_events::FunctionExecutionKey;
use ulid::Ulid;

use crate::{error::LayerDbResult, layer_cache::LayerCache, LayerDbError};

const KEYWORD_SINGULAR: &str = "function_execution";
const KEYWORD_PLURAL: &str = "function_executions";

pub const PARTITION_KEY: &str = KEYWORD_PLURAL;
pub const DBNAME: &str = KEYWORD_PLURAL;
pub const CACHE_NAME: &str = KEYWORD_PLURAL;
pub const SORT_KEY: &str = KEYWORD_SINGULAR;

#[derive(Debug, Clone)]
pub struct FunctionExecutionDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub cache: LayerCache<Arc<V>>,
}

impl<V> FunctionExecutionDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(cache: LayerCache<Arc<V>>) -> Self {
        FunctionExecutionDb { cache }
    }

    pub async fn write(&self, key: FunctionExecutionKey, value: Arc<V>) -> LayerDbResult<()> {
        let postcard_value = postcard::to_stdvec(&value)?;

        self.cache
            .pg()
            .insert(key.value(), SORT_KEY, &postcard_value)
            .await?;

        Ok(())
    }

    pub async fn read(&self, key: &FunctionExecutionKey) -> LayerDbResult<Option<Arc<V>>> {
        match self.cache.pg().get(key.value()).await? {
            Some(value) => {
                let deserialized: V = postcard::from_bytes(&value)?;
                Ok(Some(deserialized.into()))
            }
            None => Ok(None),
        }
    }

    /// We often need to extract the value from the arc by cloning it (although
    /// this should be avoided for large values). This will do that, and also
    /// helpfully convert the value to the type we want to deal with
    pub async fn try_read_as<T>(&self, key: &FunctionExecutionKey) -> LayerDbResult<Option<T>>
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
        keys: &[FunctionExecutionKey],
    ) -> LayerDbResult<HashMap<FunctionExecutionKey, Arc<V>>> {
        let mut formatted_keys = vec![];
        for key in keys {
            let key_str: Arc<str> = key.value().to_string().into();
            formatted_keys.push(key_str);
        }
        let mut values: HashMap<FunctionExecutionKey, Arc<V>> = HashMap::new();
        if let Some(found) = self.cache.pg().get_many(&formatted_keys).await? {
            for (k, v) in found {
                values.insert(
                    k.parse().map_err(|err: ParseError| {
                        LayerDbError::CouldNotConvertToKeyFromString(err.to_string())
                    })?,
                    postcard::from_bytes(&v)?,
                );
            }
        }
        Ok(values)
    }

    pub async fn read_many_by_component_id(
        &self,
        component_id: &Ulid,
    ) -> LayerDbResult<HashMap<FunctionExecutionKey, Arc<V>>> {
        self.read_many_by_prefix(&component_id.to_string()).await
    }

    pub async fn read_many_by_prototype_id(
        &self,
        prototype_id: &Ulid,
    ) -> LayerDbResult<HashMap<FunctionExecutionKey, Arc<V>>> {
        self.read_many_by_prefix(&format!("%{}", &prototype_id.to_string()))
            .await
    }

    pub async fn read_many_by_component_id_and_prototype_id(
        &self,
        component_id: &Ulid,
        prototype_id: &Ulid,
    ) -> LayerDbResult<HashMap<FunctionExecutionKey, Arc<V>>> {
        self.read_many_by_prefix(&format!(
            "{}{}",
            &component_id.to_string(),
            &prototype_id.to_string()
        ))
        .await
    }

    async fn read_many_by_prefix(
        &self,
        key: &str,
    ) -> LayerDbResult<HashMap<FunctionExecutionKey, Arc<V>>> {
        let mut values: HashMap<FunctionExecutionKey, Arc<V>> = HashMap::new();
        if let Some(found) = self.cache.pg().get_many_by_prefix(key).await? {
            for (k, v) in found {
                values.insert(
                    k.parse().map_err(|err: ParseError| {
                        LayerDbError::CouldNotConvertToKeyFromString(err.to_string())
                    })?,
                    postcard::from_bytes(&v)?,
                );
            }
        }
        Ok(values)
    }
}

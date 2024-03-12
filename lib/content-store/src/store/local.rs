use crate::store::{Store, StoreResult};
use crate::ContentHash;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use telemetry::prelude::*;

/// A kind of content store that operates entirely in memory.
#[derive(Default, Debug)]
pub struct LocalStore(HashMap<ContentHash, Vec<u8>>);

#[async_trait::async_trait]
impl Store for LocalStore {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn add<T>(&mut self, object: &T) -> StoreResult<ContentHash>
    where
        T: Serialize + ?Sized,
    {
        let value = serde_json::to_vec(object)?;
        let key = ContentHash::new(&value);
        self.0.insert(key, value);
        Ok(key)
    }

    async fn get<T>(&mut self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let maybe_object = match self.0.get(key) {
            Some(value) => Some(serde_json::from_slice(value)?),
            None => None,
        };
        Ok(maybe_object)
    }

    async fn get_bulk<T>(&mut self, keys: &[ContentHash]) -> StoreResult<HashMap<ContentHash, T>>
    where
        T: DeserializeOwned + std::marker::Send,
    {
        Ok(keys
            .iter()
            .filter_map(|key| match self.0.get(key) {
                None => None,
                Some(item) => match serde_json::from_slice(item) {
                    Ok(deserialized) => Some((key.to_owned(), deserialized)),
                    Err(err) => {
                        error!(
                            "Could not deserialize item {} in content store: {}",
                            key,
                            err.to_string()
                        );
                        None
                    }
                },
            })
            .collect())
    }

    /// This a "no-op" for the [`LocalStore`] since everything is handled in memory.
    async fn write(&mut self) -> StoreResult<()> {
        Ok(())
    }
}

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::PgError;
use std::collections::HashMap;
use thiserror::Error;

use crate::content::hash::ContentHash;
use crate::{DalContext, StandardModelError, Timestamp, TransactionsError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type StoreResult<T> = Result<T, StoreError>;

// TODO(nick): pull through cache with batched writes.
// Lifetime is tied to a DalContext.

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Store(HashMap<ContentHash, Value>);

impl Store {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<T>(&self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let maybe_value: Option<Value> = self.0.get(key).cloned();
        let value = match maybe_value {
            Some(found_value) => Some(serde_json::from_value(found_value)?),
            None => None,
        };
        Ok(value)
    }

    pub fn add<T>(&mut self, value: T) -> StoreResult<()>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(value)?;
        let hash = ContentHash::new(value.to_string().as_bytes());
        self.0.insert(hash, value);
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn add() {
//         let mut store = Store::new();
//         store.add_raw("poop").expect("could not add item");
//
//         let hash = ContentHash::new("poop".as_bytes());
//         let found_value = store.get(&hash).expect("could not get item");
//
//         assert_eq!(
//             value,       // expected
//             found_value  // actual
//         );
//     }
// }

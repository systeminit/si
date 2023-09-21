use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::content::hash::ContentHash;
use crate::DalContext;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type StoreResult<T> = Result<T, StoreError>;

pub trait Store {
    fn new() -> Self;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn add<T>(&mut self, value: T) -> StoreResult<(ContentHash, bool)>;
    fn get<T>(&self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned;
    fn write(&mut self) -> StoreResult<()>;
}

#[derive(Debug, Clone)]
struct StoreItem {
    value: Value,
    written: bool,
}

#[derive(Default, Debug)]
pub struct LocalStore(HashMap<ContentHash, StoreItem>);

impl Store for LocalStore {
    fn new() -> Self {
        todo!()
    }

    fn is_empty(&self) -> bool {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn add<T>(&mut self, value: T) -> StoreResult<(ContentHash, bool)> {
        todo!()
    }

    fn get<T>(&self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        todo!()
    }

    fn write(&mut self) -> StoreResult<()> {
        todo!()
    }
}

impl LocalStore {
    // NOTE(nick): use local, pull through or return None.
    pub fn get<T>(&self, key: &ContentHash) -> StoreResult<Option<T>>
    where
        T: DeserializeOwned,
    {
        let maybe_item: Option<StoreItem> = self.inner.get(key).cloned();
        let value = match maybe_item {
            Some(found_item) => Some(serde_json::from_value(found_item.value)?),
            None => {
                // TODO(nick): either populate from database ("pull-through caching") or return None.
                None
            }
        };
        Ok(value)
    }

    // TODO(nick): actually do stuff with the database.
    pub fn write(&mut self) -> StoreResult<()> {
        for item in self.inner.values_mut() {
            if !item.written {
                // TODO(nick): perform find or create in the database. Either way, we need to
                // set "processed" to true for the next time we perform a batch write.

                item.written = true;
            }
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct FullStore<'ctx> {
    inner: HashMap<ContentHash, StoreItem>,
    ctx: &'ctx DalContext,
}

// impl Store {
//     pub fn new() -> Self {
//         Self::default()
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.inner.is_empty()
//     }
//
//     pub fn len(&self) -> usize {
//         self.inner.len()
//     }
//
//     // NOTE(nick): existing entries must remain immutable.
//     pub fn add<T>(&mut self, value: T) -> StoreResult<(ContentHash, bool)>
//     where
//         T: Serialize + ToOwned,
//     {
//         let value = serde_json::to_value(value)?;
//         let hash = ContentHash::from(&value);
//         let already_in_store = self.inner.contains_key(&hash);
//         if !already_in_store {
//             // NOTE(nick): we DO NOT check that it is in the database because it does not matter.
//             // We wait until write time to talk to the database.
//             self.inner.insert(
//                 hash,
//                 StoreItem {
//                     value,
//                     written: false,
//                 },
//             );
//         }
//         Ok((hash, already_in_store))
//     }
//
//     // NOTE(nick): use local, pull through or return None.
//     pub fn get<T>(&self, key: &ContentHash) -> StoreResult<Option<T>>
//     where
//         T: DeserializeOwned,
//     {
//         let maybe_item: Option<StoreItem> = self.inner.get(key).cloned();
//         let value = match maybe_item {
//             Some(found_item) => Some(serde_json::from_value(found_item.value)?),
//             None => {
//                 // TODO(nick): either populate from database ("pull-through caching") or return None.
//                 None
//             }
//         };
//         Ok(value)
//     }
//
//     // TODO(nick): actually do stuff with the database.
//     pub fn write(&mut self, ctx: Option<&DalContext>) -> StoreResult<()> {
//         for item in self.inner.values_mut() {
//             if !item.written {
//                 // TODO(nick): perform find or create in the database. Either way, we need to
//                 // set "processed" to true for the next time we perform a batch write.
//
//                 item.written = true;
//             }
//         }
//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let mut store = Store::new();

        // Add an item.
        let sirens_value = "SIRENS".to_string();
        let (sirens_hash, already_in_store) = store.add(&sirens_value).expect("could not add item");
        assert!(!already_in_store);

        // Grab the value from the store and perform the assertion.
        let found_sirens_value: String = store
            .get(&sirens_hash)
            .expect("could not get item")
            .expect("no item found");
        assert_eq!(
            sirens_value,       // expected
            found_sirens_value  // actual
        );
        assert_eq!(
            1,           // expected
            store.len()  // actual
        );

        // Add another item.
        let meltdown_value = "MELTDOWN".to_string();
        let (meltdown_hash, _) = store.add(&meltdown_value).expect("could not add item");
        assert!(!already_in_store);

        // Check both entries to ensure that nothing has drifted.
        let found_meltdown_value: String = store
            .get(&meltdown_hash)
            .expect("could not get item")
            .expect("no item found");
        assert_eq!(
            meltdown_value,       // expected
            found_meltdown_value  // actual
        );
        let found_sirens_value: String = store
            .get(&sirens_hash)
            .expect("could not get item")
            .expect("no item found");
        assert_eq!(
            sirens_value,       // expected
            found_sirens_value  // actual
        );
        assert_eq!(
            2,           // expected
            store.len()  // actual
        );

        // Try to add one of the items again and check if it already exists.
        let (second_meltdown_hash, already_in_store) =
            store.add(&meltdown_value).expect("could not add item");
        assert!(already_in_store);
        assert_eq!(
            meltdown_hash,        // expected
            second_meltdown_hash, // actual
        )
    }

    #[test]
    fn write() {
        let mut store = Store::new();

        // Populate the store and then write.
        for value in ["PARASAIL", "TELEKINESIS"] {
            let (_, already_in_store) = store.add(value).expect("could not add item");
            assert!(!already_in_store);
        }

        // Since purely "adding" does not involve the database, none of our entries known if they
        // were processed.
        for item in store.inner.values() {
            assert!(!item.written);
        }

        // FIXME(nick): once write actually talks to the database, this will need to move to an
        // integration test. Check that all items have been processed.
        store.write(None).expect("could not write");
        for item in store.inner.values() {
            assert!(item.written);
        }

        // Add another item.
        let (utopia_hash, already_in_store) = store.add("UTOPIA").expect("could not add item");
        assert!(!already_in_store);

        // Check that only the new item has not been processed and that all other items have been
        // processed.
        for (hash, item) in &store.inner {
            assert_eq!(hash != &utopia_hash, item.written);
        }

        // Write again and assert all items have been processed.
        store.write(None).expect("could not write");
        for item in store.inner.values() {
            assert!(item.written);
        }
    }
}

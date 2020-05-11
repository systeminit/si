use std::future::Future;
use std::pin::Pin;

use couchbase::{options::ReplaceOptions, CouchbaseError};
use serde::{Deserialize, Serialize};

use crate::db::Db;
use crate::error::{DataError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
// A globally unique transaction id. It is always stored as a single
// document, which we look up, and then perform an atomic compare and
// swap on it. If we fail, we retry, until we get a unique number.
//
// This code is probably hyper agressive.
pub struct TxnId {
    pub counter: u128,
}

impl TxnId {
    pub fn get(db: Db) -> Pin<Box<dyn Future<Output = Result<TxnId>> + Send + Sync>> {
        Box::pin(async move {
            let get_result = db
                .bucket
                .default_collection()
                .get("global_txn_id", None)
                .await;
            let (mut txn_id, cas) = match get_result {
                Err(CouchbaseError::KeyDoesNotExist) => {
                    let txn_id = TxnId { counter: 1 };
                    match db
                        .bucket
                        .default_collection()
                        .insert("global_txn_id", txn_id.clone(), None)
                        .await
                    {
                        Err(CouchbaseError::KeyExists) => {
                            return TxnId::get(db).await;
                        }
                        Err(err) => return Err(DataError::CouchbaseError(err)),
                        Ok(item) => {
                            let cas = item.cas();
                            (txn_id, cas)
                        }
                    }
                }
                Err(err) => return Err(DataError::CouchbaseError(err)),
                Ok(item) => {
                    let txn_id: TxnId = item.content_as()?;
                    let cas = item.cas();
                    (txn_id, cas)
                }
            };
            txn_id.increment();
            match db
                .bucket
                .default_collection()
                .replace(
                    "global_txn_id",
                    txn_id.clone(),
                    Some(ReplaceOptions::new().set_cas(cas)),
                )
                .await
            {
                Ok(_) => return Ok(txn_id),
                Err(CouchbaseError::KeyExists) => {
                    return TxnId::get(db).await;
                }
                Err(err) => return Err(DataError::CouchbaseError(err)),
            };
        })
    }

    fn increment(&mut self) {
        self.counter = self.counter + 1;
    }

    pub fn value(&self) -> u128 {
        self.counter
    }
}

impl std::convert::TryFrom<String> for TxnId {
    type Error = DataError;

    fn try_from(value: String) -> Result<Self> {
        let counter: u128 = value.parse()?;
        Ok(TxnId { counter })
    }
}

impl std::fmt::Display for TxnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.counter)
    }
}

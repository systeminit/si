use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use si_data::PgError;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LabelListError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
}

pub type LabelListResult<T> = Result<T, LabelListError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct LabelEntry<Value> {
    pub label: String,
    pub value: Value,
}

impl<Value: Debug + Serialize + DeserializeOwned> LabelEntry<Value> {
    pub fn new(label: impl Into<String>, value: Value) -> Self {
        let label = label.into();
        LabelEntry { label, value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct LabelList<Value>(Vec<LabelEntry<Value>>);

impl<Value: Debug + DeserializeOwned + Serialize + postgres_types::FromSqlOwned> LabelList<Value> {
    pub fn from_rows(rows: Vec<tokio_postgres::Row>) -> LabelListResult<LabelList<Value>> {
        let mut results = Vec::new();
        for row in rows.into_iter() {
            let name: String = row.try_get("name")?;
            let value: Value = row.try_get("value")?;
            results.push(LabelEntry::new(name, value));
        }
        Ok(LabelList(results))
    }
}

impl<Value: DeserializeOwned + Serialize + Debug> Deref for LabelList<Value> {
    type Target = Vec<LabelEntry<Value>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Value: DeserializeOwned + Serialize + Debug> DerefMut for LabelList<Value> {
    fn deref_mut(&mut self) -> &mut Vec<LabelEntry<Value>> {
        &mut self.0
    }
}

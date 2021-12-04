use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use si_data::PgError;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use strum::IntoEnumIterator;
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
pub struct LabelEntry<V> {
    pub label: String,
    pub value: V,
}

impl<V> LabelEntry<V> {
    pub fn new(label: impl Into<String>, value: V) -> Self {
        let label = label.into();
        LabelEntry { label, value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct LabelList<V>(Vec<LabelEntry<V>>);

impl<V> LabelList<V> {
    pub fn new(options: Vec<LabelEntry<V>>) -> Self {
        LabelList(options)
    }
}

impl<V> From<Vec<LabelEntry<V>>> for LabelList<V> {
    fn from(value: Vec<LabelEntry<V>>) -> Self {
        Self(value)
    }
}

impl<V> LabelList<V>
where
    V: Debug + DeserializeOwned + Serialize + postgres_types::FromSqlOwned,
{
    pub fn from_rows(rows: Vec<tokio_postgres::Row>) -> LabelListResult<LabelList<V>> {
        let mut results = Vec::new();
        for row in rows.into_iter() {
            let name: String = row.try_get("name")?;
            let value: V = row.try_get("value")?;
            results.push(LabelEntry::new(name, value));
        }
        Ok(LabelList(results))
    }
}

impl<V> Deref for LabelList<V> {
    type Target = Vec<LabelEntry<V>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for LabelList<V> {
    fn deref_mut(&mut self) -> &mut Vec<LabelEntry<V>> {
        &mut self.0
    }
}

pub trait ToLabelList: IntoEnumIterator + Serialize + ToString {
    fn to_label_list() -> std::result::Result<LabelList<serde_json::Value>, LabelListError> {
        let mut list = Vec::new();
        for v in Self::iter() {
            list.push(LabelEntry::new(v.to_string(), serde_json::to_value(v)?));
        }
        Ok(list.into())
    }
}

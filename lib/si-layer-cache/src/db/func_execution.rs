use std::sync::Arc;

use serde::{de::DeserializeOwned, Serialize};
use si_events::ulid::Ulid;
use si_events::{ContentHash, FuncExecutionKey, FuncExecutionMessage};

use super::serialize;
use crate::{error::LayerDbResult, layer_cache::LayerCache, LayerDbError};

const KEYWORD_SINGULAR: &str = "func_execution";
const KEYWORD_PLURAL: &str = "func_executions";

pub const PARTITION_KEY: &str = KEYWORD_PLURAL;
pub const DBNAME: &str = KEYWORD_PLURAL;
pub const CACHE_NAME: &str = KEYWORD_PLURAL;
pub const SORT_KEY: &str = KEYWORD_SINGULAR;
pub const EXECUTIONS_TABLE_NAME: &str = KEYWORD_PLURAL;
pub const MESSAGES_TABLE_NAME: &str = "func_execution_messages";

#[derive(Debug, Clone)]
pub struct FuncExecutionDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub cache: LayerCache<Arc<V>>,
    attach_message_query: String,
    get_many_func_execution_by_id_query: String,
    get_many_func_execution_by_where_query: String,
    get_message_value_query: String,
    insert_execution_query: String,
    insert_message_query: String,
}

impl<V> FuncExecutionDb<V>
where
    V: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    pub fn new(cache: LayerCache<Arc<V>>) -> Self {
        FuncExecutionDb { cache ,
            attach_message_query: format!("UPDATE {EXECUTIONS_TABLE_NAME} SET message_id = $1 WHERE key = $2"),
            get_many_func_execution_by_id_query: format!("SELECT * FROM {EXECUTIONS_TABLE_NAME} WHERE key = any($1)"),
            get_many_func_execution_by_where_query: format!("SELECT * FROM {EXECUTIONS_TABLE_NAME} WHERE "),
            get_message_value_query: format!("SELECT value FROM {MESSAGES_TABLE_NAME} WHERE key = $1 LIMIT 1"),
            insert_execution_query: format!("INSERT INTO {EXECUTIONS_TABLE_NAME} (key, action_id, component_id, prototype_id, value) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING"),
            insert_message_query: format!("INSERT INTO {MESSAGES_TABLE_NAME} (key, sort_key, value) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"),
        }
    }

    // writes a [`FuncExecution`] to the database
    pub async fn write(
        &self,
        mut key: FuncExecutionKey,
        value: V,
    ) -> LayerDbResult<FuncExecutionKey> {
        let postcard_value = serialize::to_vec(&value)?;
        let hash = ContentHash::new(&postcard_value);
        self.cache
            .pg()
            .insert_raw(
                &self.insert_execution_query,
                &[
                    &hash,
                    &key.action_id().to_string(),
                    &key.component_id().to_string(),
                    &key.prototype_id().to_string(),
                    &postcard_value,
                ],
            )
            .await?;
        key.func_execution_id = Some(hash);
        Ok(key)
    }

    // writes a [`FuncExecutionMessage`] to the database and then updates the relevant
    // [`FunctionExecution`] to reference it
    pub async fn write_message(
        &self,
        mut key: FuncExecutionKey,
        value: FuncExecutionMessage,
    ) -> LayerDbResult<FuncExecutionKey> {
        if let Some(func_execution_id) = key.func_execution_id() {
            let value = serialize::to_vec(&value)?;
            let hash = ContentHash::new(&value);

            self.cache
                .pg()
                .insert_raw(&self.insert_message_query, &[&hash, &SORT_KEY, &value])
                .await?;

            self.cache
                .pg()
                .insert_raw(&self.attach_message_query, &[&hash, func_execution_id])
                .await?;

            key.message_id = Some(hash);
            Ok(key)
        } else {
            Err(LayerDbError::IncompleteKey(
                "missing func_execution_id".to_string(),
            ))
        }
    }

    // reads a [`FuncExecution`] from the database
    pub async fn read(&self, key: FuncExecutionKey) -> LayerDbResult<Option<V>> {
        Ok(match key.func_execution_id() {
            Some(key) => match self.cache.pg().get(&key.to_string()).await? {
                Some(value) => Some(serialize::from_bytes(&value)?),
                None => None,
            },
            None => None,
        })
    }

    // reads a [`FuncExecutionMessage`] from the database. Note that the [`FuncExecutionKey`] being
    // supplied MUST include the message_id for retrieval
    pub async fn read_message(
        &self,
        key: FuncExecutionKey,
    ) -> LayerDbResult<Option<FuncExecutionMessage>> {
        Ok(if let Some(message_id) = key.message_id() {
            match self
                .cache
                .pg()
                .get_raw(&self.get_message_value_query, &[&message_id])
                .await?
            {
                Some(row) => {
                    let deserialized: FuncExecutionMessage =
                        serialize::from_bytes(row.get("value"))?;
                    Some(deserialized)
                }
                None => None,
            }
        } else {
            return Err(LayerDbError::IncompleteKey(
                "missing message_id".to_string(),
            ));
        })
    }

    // reads all [`FuncExecution`]s from the database. Note that the [`FuncExecutionKey`]s being
    // supplied MUST include the func_execution_id for retrieval
    pub async fn read_many(&self, keys: &[FuncExecutionKey]) -> LayerDbResult<Option<Vec<V>>> {
        let func_keys: Vec<&ContentHash> = keys
            .iter()
            .filter_map(|key| key.func_execution_id())
            .collect();

        Ok(self
            .cache
            .pg()
            .get_many_raw(&self.get_many_func_execution_by_id_query, &[&func_keys])
            .await?
            .map(|rows| {
                rows.iter()
                    .map(|row| serialize::from_bytes(row.get("value")))
                    .filter_map(Result::ok)
                    .collect()
            }))
    }

    async fn read_many_by_where(&self, where_clause: &str) -> LayerDbResult<Option<Vec<V>>> {
        Ok(self
            .cache
            .pg()
            .get_many_raw(
                &format!(
                    "{}{}",
                    &self.get_many_func_execution_by_where_query, where_clause
                ),
                &[],
            )
            .await?
            .map(|rows| {
                rows.iter()
                    .map(|row| serialize::from_bytes(row.get("value")))
                    .filter_map(Result::ok)
                    .collect()
            }))
    }

    // reads all [`FuncExecution`]s from the database for a given component_id
    pub async fn read_many_by_component_id(
        &self,
        component_id: &Ulid,
    ) -> LayerDbResult<Option<Vec<V>>> {
        self.read_many_by_where(&format!("component_id = '{}'", component_id))
            .await
    }

    // reads all [`FuncExecution`]s from the database for a given prototype_id
    pub async fn read_many_by_prototype_id(
        &self,
        prototype_id: &Ulid,
    ) -> LayerDbResult<Option<Vec<V>>> {
        self.read_many_by_where(&format!("prototype_id = '{}'", prototype_id))
            .await
    }

    // reads all [`FuncExecution`]s from the database for a given component_id + prototype_id
    pub async fn read_many_by_component_id_and_prototype_id(
        &self,
        component_id: &Ulid,
        prototype_id: &Ulid,
    ) -> LayerDbResult<Option<Vec<V>>> {
        self.read_many_by_where(&format!(
            "component_id = '{}' AND prototype_id = '{}'",
            component_id, prototype_id,
        ))
        .await
    }
}

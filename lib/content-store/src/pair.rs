use crate::hash::ContentHash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::{PgError, PgPool, PgPoolError};
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ContentPairError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type ContentPairResult<T> = Result<T, ContentPairError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentPair {
    key: String,
    created_at: DateTime<Utc>,
    value: Value,
}

impl ContentPair {
    pub async fn find_or_create(
        pg_pool: &PgPool,
        key: ContentHash,
        value: Value,
    ) -> ContentPairResult<(Self, bool)> {
        let (pair, created): (Self, bool) = match Self::find(pg_pool, &key).await? {
            Some(found) => (found, false),
            None => {
                let client = pg_pool.get().await?;
                let row = client
                    .query_one(
                        "INSERT INTO content_pairs (key, value) VALUES ($1, $2) AS object",
                        &[&key.to_string(), &value],
                    )
                    .await?;
                let json: Value = row.try_get("object")?;
                (serde_json::from_value(json)?, true)
            }
        };
        Ok((pair, created))
    }

    pub async fn find(pg_pool: &PgPool, key: &ContentHash) -> ContentPairResult<Option<Self>> {
        let client = pg_pool.get().await?;
        let maybe_row = client
            .query_opt(
                "SELECT * FROM content_pairs WHERE key = $1 AS object",
                &[&key.to_string()],
            )
            .await?;
        let result = match maybe_row {
            Some(found_row) => {
                let json: Value = found_row.try_get("object")?;
                let object: Self = serde_json::from_value(json)?;
                Some(object)
            }
            None => None,
        };
        Ok(result)
    }
}

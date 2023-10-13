use crate::hash::ContentHash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgPool, PgPoolError, PgRow};
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

pub(crate) type ContentPairResult<T> = Result<T, ContentPairError>;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ContentPair {
    key: String,
    created_at: DateTime<Utc>,
    /// Serialized CBOR bytes.
    value: Vec<u8>,
}

impl TryFrom<PgRow> for ContentPair {
    type Error = ContentPairError;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        Ok(Self {
            key: row.try_get("key")?,
            created_at: row.try_get("created_at")?,
            value: row.try_get("value")?,
        })
    }
}

impl ContentPair {
    pub(crate) fn value(&self) -> &[u8] {
        &self.value
    }

    pub(crate) async fn find_or_create(
        pg_pool: &PgPool,
        key: ContentHash,
        value: Vec<u8>,
    ) -> ContentPairResult<Self> {
        let content_pair = match Self::find(pg_pool, &key).await? {
            Some(found_content_pair) => found_content_pair,
            None => {
                let client = pg_pool.get().await?;
                let row = client
                    .query_one(
                        "INSERT INTO content_pairs (key, value) VALUES ($1, $2) RETURNING *",
                        &[&key.to_string(), &value],
                    )
                    .await?;
                Self::try_from(row)?
            }
        };
        Ok(content_pair)
    }

    pub(crate) async fn find(
        pg_pool: &PgPool,
        key: &ContentHash,
    ) -> ContentPairResult<Option<Self>> {
        let client = pg_pool.get().await?;
        let maybe_row = client
            .query_opt(
                "SELECT * FROM content_pairs WHERE key = $1",
                &[&key.to_string()],
            )
            .await?;
        match maybe_row {
            Some(row) => Ok(Some(Self::try_from(row)?)),
            None => Ok(None),
        }
    }
}

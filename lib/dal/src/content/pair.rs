use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::PgError;
use thiserror::Error;

use crate::content::hash::ContentHash;
use crate::{DalContext, StandardModelError, Timestamp, TransactionsError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ContentPairError {
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type ContentPairResult<T> = Result<T, ContentPairError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentPair {
    #[serde(flatten)]
    timestamp: Timestamp,
    key: String,
    value: Value,
}

impl ContentPair {
    pub async fn find_or_create(
        ctx: &DalContext,
        key: ContentHash,
        value: Value,
    ) -> ContentPairResult<(Self, bool)> {
        let (pair, created): (Self, bool) = match Self::find(ctx, &key).await? {
            Some(found) => (found, false),
            None => {
                let row = ctx
                    .txns()
                    .await?
                    .pg()
                    .query_one(
                        "SELECT content_pair_create_v1($1) AS object",
                        &[&key.to_string(), &value],
                    )
                    .await?;
                let json: Value = row.try_get("object")?;
                (serde_json::from_value(json)?, true)
            }
        };
        Ok((pair, created))
    }

    pub async fn find(ctx: &DalContext, key: &ContentHash) -> ContentPairResult<Option<Self>> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
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

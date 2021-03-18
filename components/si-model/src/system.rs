use si_data::PgTxn;
use thiserror::Error;

use crate::{EntityError, LabelList, LabelListItem};

const SYSTEM_LIST_AS_LABELS: &str = include_str!("./queries/system_list_as_labels.sql");

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
}

pub type SystemResult<T> = Result<T, SystemError>;

pub async fn list_as_labels(
    txn: &PgTxn<'_>,
    workspace_id: impl AsRef<str>,
) -> SystemResult<LabelList> {
    let workspace_id = workspace_id.as_ref();
    let mut results = Vec::new();
    let rows = txn.query(SYSTEM_LIST_AS_LABELS, &[&workspace_id]).await?;
    for row in rows.into_iter() {
        let json: serde_json::Value = row.try_get("item")?;
        let object: LabelListItem = serde_json::from_value(json)?;
        results.push(object);
    }

    return Ok(results);
}

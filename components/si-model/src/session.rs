use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::PgTxn;

use crate::{Entity, Organization, Workspace};

const GET_DEFAULTS: &str = include_str!("./queries/session_dal_get_defaults.sql");

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type SessionResult<T> = Result<T, SessionError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionDefaults {
    pub organization: Organization,
    pub workspace: Workspace,
    pub system: Entity,
}

pub async fn get_defaults(
    txn: &PgTxn<'_>,
    billing_account_id: impl AsRef<str>,
) -> SessionResult<SessionDefaults> {
    let billing_account_id = billing_account_id.as_ref();
    let row = txn.query_one(GET_DEFAULTS, &[&billing_account_id]).await?;

    let org_json: serde_json::Value = row.try_get("organization")?;
    let organization: Organization = serde_json::from_value(org_json)?;
    let w_json: serde_json::Value = row.try_get("workspace")?;
    let workspace: Workspace = serde_json::from_value(w_json)?;
    let s_json: serde_json::Value = row.try_get("system")?;
    let system: Entity = serde_json::from_value(s_json)?;

    Ok(SessionDefaults {
        organization,
        workspace,
        system,
    })
}

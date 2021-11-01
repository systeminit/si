use rand::Rng;
use thiserror::Error;
use tracing::instrument;

pub mod billing_account;
pub mod change_set;
pub mod edit_session;
pub mod history_event;
pub mod jwt_key;
pub mod standard_accessors;
pub mod standard_model;
pub mod standard_pk;
pub mod tenancy;
pub mod test_harness;
pub mod timestamp;
pub mod visibility;

pub use billing_account::{BillingAccount, BillingAccountError};
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetPk, ChangeSetStatus, NO_CHANGE_SET_PK};
pub use edit_session::{
    EditSession, EditSessionError, EditSessionPk, EditSessionStatus, NO_EDIT_SESSION_PK,
};
pub use history_event::{HistoryActor, HistoryEvent, HistoryEventError};
pub use jwt_key::create_jwt_key_if_missing;
pub use standard_model::{StandardModel, StandardModelError, StandardModelResult};
pub use tenancy::{Tenancy, TenancyError};
pub use timestamp::{Timestamp, TimestampError};
pub use visibility::{Visibility, VisibilityError};

use si_data::{NatsConn, NatsTxnError, PgError, PgPool, PgPoolError};

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}

const NAME_CHARSET: &[u8] = b"0123456789";

#[derive(Error, Debug)]
pub enum ModelError {
    #[error(transparent)]
    Migration(#[from] PgPoolError),
    #[error(transparent)]
    NatsTxnError(#[from] NatsTxnError),
    #[error("database error")]
    PgError(#[from] PgError),
}

pub type ModelResult<T> = Result<T, ModelError>;

#[instrument(skip_all)]
pub async fn migrate(pg: &PgPool) -> ModelResult<()> {
    let result = pg.migrate(embedded::migrations::runner()).await?;
    Ok(result)
}

#[instrument(skip(pg, nats))]
pub async fn migrate_builtin_schemas(pg: &PgPool, nats: &NatsConn) -> ModelResult<()> {
    let mut conn = pg.get().await?;
    let txn = conn.transaction().await?;
    let nats = nats.transaction();
    //schema_builtins::system::migrate(&txn, &nats).await?;
    txn.commit().await?;
    nats.commit().await?;
    Ok(())
}

pub fn generate_name(name: Option<String>) -> String {
    if name.is_some() {
        return name.unwrap();
    }
    let mut rng = rand::thread_rng();
    let unique_id: String = (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..NAME_CHARSET.len());
            NAME_CHARSET[idx] as char
        })
        .collect();
    return format!("si-{}", unique_id);
}

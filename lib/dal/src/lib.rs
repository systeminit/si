use rand::Rng;
use si_data::{NatsClient, NatsError, PgError, PgPool, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;

pub mod billing_account;
pub mod capability;
pub mod change_set;
pub mod component;
pub mod edit_field;
pub mod edit_session;
pub mod group;
pub mod history_event;
pub mod jwt_key;
pub mod key_pair;
pub mod label_list;
pub mod node;
pub mod organization;
pub mod qualification_check;
pub mod schema;
pub mod schematic;
pub mod socket;
pub mod standard_accessors;
pub mod standard_model;
pub mod standard_pk;
pub mod system;
pub mod tenancy;
pub mod test_harness;
pub mod timestamp;
pub mod user;
pub mod visibility;
pub mod workspace;
pub mod ws_event;

pub use billing_account::{
    BillingAccount, BillingAccountDefaults, BillingAccountError, BillingAccountId, BillingAccountPk,
};
pub use capability::{Capability, CapabilityError, CapabilityId, CapabilityPk, CapabilityResult};
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetPk, ChangeSetStatus, NO_CHANGE_SET_PK};
pub use component::{Component, ComponentError, ComponentId};
pub use edit_session::{
    EditSession, EditSessionError, EditSessionPk, EditSessionStatus, NO_EDIT_SESSION_PK,
};
pub use group::{Group, GroupError, GroupId, GroupResult};
pub use history_event::{HistoryActor, HistoryEvent, HistoryEventError};
pub use jwt_key::{create_jwt_key_if_missing, JwtSecretKey};
pub use key_pair::{KeyPair, KeyPairError, KeyPairResult};
pub use label_list::{LabelEntry, LabelList, LabelListError};
pub use node::{Node, NodeError};
pub use organization::{
    Organization, OrganizationError, OrganizationId, OrganizationPk, OrganizationResult,
};
pub use qualification_check::{
    QualificationCheck, QualificationCheckError, QualificationCheckId, QualificationCheckPk,
};
pub use schema::{
    Schema, SchemaError, SchemaId, SchemaKind, SchemaPk, SchemaVariant, SchemaVariantId,
};
pub use schematic::SchematicKind;
pub use standard_model::{StandardModel, StandardModelError, StandardModelResult};
pub use system::{System, SystemError, SystemId, SystemPk, SystemResult};
pub use tenancy::{Tenancy, TenancyError};
pub use timestamp::{Timestamp, TimestampError};
pub use user::{User, UserClaim, UserError, UserId, UserResult};
pub use visibility::{Visibility, VisibilityError};
pub use workspace::{Workspace, WorkspaceError, WorkspaceId, WorkspacePk, WorkspaceResult};
pub use ws_event::{WsEvent, WsEventError, WsPayload};

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
    Nats(#[from] NatsError),
    #[error("database error")]
    PgError(#[from] PgError),
    #[error("Schema error")]
    Schema(#[from] SchemaError),
}

pub type ModelResult<T> = Result<T, ModelError>;

#[instrument(skip_all)]
pub async fn migrate(pg: &PgPool) -> ModelResult<()> {
    let result = pg.migrate(embedded::migrations::runner()).await?;
    Ok(result)
}

#[instrument(skip_all)]
pub async fn migrate_builtin_schemas(pg: &PgPool, nats: &NatsClient) -> ModelResult<()> {
    let mut conn = pg.get().await?;
    let txn = conn.transaction().await?;
    let nats = nats.transaction();
    schema::builtins::migrate(&txn, &nats).await?;
    txn.commit().await?;
    nats.commit().await?;
    Ok(())
}

pub fn generate_name(name: Option<String>) -> String {
    if let Some(name) = name {
        return name;
    }

    let mut rng = rand::thread_rng();
    let unique_id: String = (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..NAME_CHARSET.len());
            NAME_CHARSET[idx] as char
        })
        .collect();
    format!("si-{}", unique_id)
}

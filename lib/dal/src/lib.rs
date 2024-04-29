//! The Data Access Layer (DAL) for System Initiative.

#![warn(
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used
)]

use std::time::Duration;

use rand::Rng;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use si_data_nats::NatsError;
use si_data_pg::{PgError, PgPool, PgPoolError};
use strum::{Display, EnumString, VariantNames};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time;
use tokio::time::Instant;

pub mod action;
pub mod actor_view;
pub mod attribute;
pub mod authentication_prototype;
pub mod builtins;
pub mod change_set;
pub mod change_status;
pub mod code_view;
pub mod component;
pub mod context;
pub mod deprecated_action;
pub mod diagram;
pub mod func;
pub mod history_event;
pub mod input_sources;
pub mod job;
pub mod job_failure;
pub mod jwt_key;
pub mod key_pair;
pub mod label_list;
pub mod layer_db_types;
pub mod module;
pub mod pkg;
pub mod prop;
pub mod property_editor;
pub mod qualification;
pub mod schema;
pub mod secret;
pub mod serde_impls;
pub mod socket;
pub mod standard_accessors;
pub mod standard_connection;
pub mod standard_id;
pub mod standard_model;
pub mod standard_pk;
pub mod status;
pub mod tenancy;
pub mod timestamp;
pub mod user;
pub mod validation;
pub mod visibility;
pub mod workspace;
pub mod workspace_snapshot;
pub mod ws_event;

pub use action::ActionPrototypeId;
pub use actor_view::ActorView;
pub use attribute::{
    prototype::{AttributePrototype, AttributePrototypeId},
    value::{AttributeValue, AttributeValueId},
};
pub use builtins::{BuiltinsError, BuiltinsResult};
pub use change_set::status::ChangeSetStatus;
pub use change_set::ChangeSetApplyError;
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetId};
pub use component::Component;
pub use component::ComponentError;
pub use component::ComponentId;
pub use context::{
    AccessBuilder, Connections, DalContext, DalContextBuilder, DalLayerDb, RequestContext,
    ServicesContext, Transactions, TransactionsError,
};
pub use deprecated_action::batch::{
    DeprecatedActionBatch, DeprecatedActionBatchError, DeprecatedActionBatchId,
};
pub use deprecated_action::prototype::{
    DeprecatedActionKind, DeprecatedActionPrototype, DeprecatedActionPrototypeError,
    DeprecatedActionPrototypeView,
};
pub use deprecated_action::runner::{
    ActionCompletionStatus, DeprecatedActionRunner, DeprecatedActionRunnerError,
    DeprecatedActionRunnerId,
};
pub use deprecated_action::{ActionId, DeprecatedAction, DeprecatedActionError};
pub use func::{
    backend::{FuncBackendKind, FuncBackendResponseType},
    Func, FuncError, FuncId,
};
pub use history_event::{HistoryActor, HistoryEvent, HistoryEventError};
pub use job::processor::{JobQueueProcessor, NatsProcessor};
pub use job_failure::{JobFailure, JobFailureError, JobFailureResult};
pub use jwt_key::JwtPublicSigningKey;
pub use key_pair::{KeyPair, KeyPairError, KeyPairResult, PublicKey};
pub use label_list::{LabelEntry, LabelList, LabelListError};
pub use prop::{Prop, PropId, PropKind};
pub use schema::variant::root_prop::component_type::ComponentType;
pub use schema::{
    variant::SchemaVariantError, Schema, SchemaError, SchemaId, SchemaVariant, SchemaVariantId,
};
pub use secret::EncryptedSecret;
pub use secret::Secret;
pub use secret::SecretAlgorithm;
pub use secret::SecretCreatedPayload;
pub use secret::SecretDefinitionView;
pub use secret::SecretDefinitionViewError;
pub use secret::SecretError;
pub use secret::SecretId;
pub use secret::SecretResult;
pub use secret::SecretUpdatedPayload;
pub use secret::SecretVersion;
pub use secret::SecretView;
pub use secret::SecretViewError;
pub use si_events::ulid::Ulid;
pub use socket::input::{InputSocket, InputSocketId};
pub use socket::output::{OutputSocket, OutputSocketId};
pub use socket::SocketArity;
pub use socket::SocketKind;
pub use standard_connection::{HelperError, HelperResult};
pub use standard_model::{StandardModel, StandardModelError, StandardModelResult};
pub use tenancy::{Tenancy, TenancyError};
pub use timestamp::{Timestamp, TimestampError};
pub use user::{User, UserClaim, UserError, UserPk, UserResult};
pub use visibility::Visibility;
pub use workspace::{Workspace, WorkspaceError, WorkspacePk, WorkspaceResult};
pub use workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
pub use workspace_snapshot::graph::WorkspaceSnapshotGraph;
pub use workspace_snapshot::{WorkspaceSnapshot, WorkspaceSnapshotError};
pub use ws_event::{WsEvent, WsEventError, WsEventResult, WsPayload};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("failed to initialize sodium oxide")]
    SodiumOxide,
}

pub type InitializationResult<T> = Result<T, InitializationError>;

/// Perform base initializations before using the `dal`.
pub fn init() -> InitializationResult<()> {
    sodiumoxide::init().map_err(|()| InitializationError::SodiumOxide)?;
    Ok(())
}

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}

const NAME_CHARSET: &[u8] = b"0123456789";

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("builtins error: {0}")]
    Builtins(#[from] BuiltinsError),
    #[error(transparent)]
    Migration(#[from] PgPoolError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error("database error")]
    PgError(#[from] PgError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
}

pub type ModelResult<T> = Result<T, ModelError>;

#[instrument(level = "info", skip_all)]
pub async fn migrate_all(services_context: &ServicesContext) -> ModelResult<()> {
    migrate(services_context.pg_pool()).await?;
    Ok(())
}

#[instrument(level = "info", skip_all)]
pub async fn migrate_all_with_progress(services_context: &ServicesContext) -> ModelResult<()> {
    let mut interval = time::interval(Duration::from_secs(5));
    let instant = Instant::now();
    let migrate_all = migrate_all(services_context);
    tokio::pin!(migrate_all);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                info!(elapsed = instant.elapsed().as_secs_f32(), "migrating");
            }
            result = &mut migrate_all  => match result {
                Ok(_) => {
                    info!(elapsed = instant.elapsed().as_secs_f32(), "migrating completed");
                    break;
                }
                Err(err) => return Err(err),
            }
        }
    }

    Ok(())
}

#[instrument(level = "info", skip_all)]
pub async fn migrate(pg: &PgPool) -> ModelResult<()> {
    pg.migrate(embedded::migrations::runner()).await?;
    Ok(())
}

pub fn generate_unique_id(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..NAME_CHARSET.len());
            NAME_CHARSET[idx] as char
        })
        .collect()
}

pub fn generate_name_from_schema_name(schema_name: impl AsRef<str>) -> String {
    let unique_id = generate_unique_id(4);
    let schema_name = schema_name.as_ref();
    format!("{schema_name} {unique_id}")
}

pub fn generate_name() -> String {
    let unique_id = generate_unique_id(4);
    format!("si-{unique_id}")
}

#[remain::sorted]
#[derive(
    Clone,
    Debug,
    DeserializeFromStr,
    Display,
    EnumString,
    VariantNames,
    Eq,
    PartialEq,
    SerializeDisplay,
)]
#[strum(serialize_all = "camelCase")]
pub enum MigrationMode {
    Run,
    RunAndQuit,
    Skip,
}

impl Default for MigrationMode {
    fn default() -> Self {
        Self::Run
    }
}

impl MigrationMode {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <MigrationMode as strum::VariantNames>::VARIANTS
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    mod migration_mode {
        use super::*;

        #[test]
        fn display() {
            assert_eq!("run", MigrationMode::Run.to_string());
            assert_eq!("runAndQuit", MigrationMode::RunAndQuit.to_string());
            assert_eq!("skip", MigrationMode::Skip.to_string());
        }

        #[test]
        fn from_str() {
            assert_eq!(MigrationMode::Run, "run".parse().expect("failed to parse"));
            assert_eq!(
                MigrationMode::RunAndQuit,
                "runAndQuit".parse().expect("failed to parse")
            );
            assert_eq!(
                MigrationMode::Skip,
                "skip".parse().expect("failed to parse")
            );
        }

        #[test]
        fn deserialize() {
            #[derive(Deserialize)]
            struct Test {
                mode: MigrationMode,
            }

            let test: Test =
                serde_json::from_str(r#"{"mode":"runAndQuit"}"#).expect("failed to deserialize");
            assert_eq!(MigrationMode::RunAndQuit, test.mode);
        }

        #[test]
        fn serialize() {
            #[derive(Serialize)]
            struct Test {
                mode: MigrationMode,
            }

            let test = serde_json::to_string(&Test {
                mode: MigrationMode::RunAndQuit,
            })
            .expect("failed to serialize");
            assert_eq!(r#"{"mode":"runAndQuit"}"#, test);
        }
    }
}

//! The Data Access Layer (DAL) for System Initiative.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use rebaser_client::Config as RebaserClientConfig;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use si_crypto::SymmetricCryptoService;
use si_data_nats::{NatsClient, NatsError};
use si_data_pg::{PgError, PgPool, PgPoolError};
use strum::{Display, EnumString, EnumVariantNames};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time;
use tokio::time::Instant;
use veritech_client::CycloneEncryptionKey;

use crate::builtins::SelectedTestBuiltinSchemas;

//pub mod action;
pub mod action_prototype;
pub mod actor_view;
pub mod attribute;
pub mod authentication_prototype;
pub mod builtins;
pub mod change_set;
pub mod change_set_pointer;
pub mod change_status;
pub mod component;
pub mod context;
pub mod diagram;
pub mod func;
pub mod history_event;
pub mod installed_pkg;
pub mod job;
pub mod job_failure;
pub mod jwt_key;
pub mod key_pair;
pub mod label_list;
pub mod pkg;
pub mod prop;
pub mod property_editor;
pub mod provider;
pub mod schema;
pub mod serde_impls;
pub mod standard_accessors;
pub mod standard_model;
pub mod standard_pk;
pub mod tenancy;
pub mod timestamp;
pub mod user;
pub mod validation;
pub mod visibility;
pub mod workspace;
pub mod workspace_snapshot;
pub mod ws_event;

// TODO(nick,jacob): this should self-destruct once the new engine is in place.
// pub mod node;
// pub mod socket;

//pub mod code_view;
// pub mod edge;
// pub mod fix;
// pub mod index_map;
pub mod node_menu;
// pub mod prop_tree;
// pub mod prototype_context;
// pub mod prototype_list_for_func;
pub mod qualification;
// pub mod reconciliation_prototype;
pub mod secret;
// pub mod status;
//pub mod tasks;

pub use action_prototype::{ActionKind, ActionPrototype, ActionPrototypeId};
pub use actor_view::ActorView;
pub use attribute::{
    prototype::{AttributePrototype, AttributePrototypeId},
    value::{AttributeValue, AttributeValueId},
};
pub use builtins::{BuiltinsError, BuiltinsResult};
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetPk, ChangeSetStatus};
pub use component::Component;
pub use component::ComponentError;
pub use component::ComponentId;
pub use component::ComponentKind;
pub use context::{
    AccessBuilder, Connections, DalContext, DalContextBuilder, RequestContext, ServicesContext,
    Transactions, TransactionsError,
};
pub use func::{
    backend::{FuncBackendKind, FuncBackendResponseType},
    Func, FuncId,
};
pub use history_event::{HistoryActor, HistoryEvent, HistoryEventError};
pub use job::processor::{JobQueueProcessor, NatsProcessor};
pub use job_failure::{JobFailure, JobFailureError, JobFailureResult};
pub use jwt_key::JwtPublicSigningKey;
pub use key_pair::{KeyPair, KeyPairError, KeyPairResult, PublicKey};
pub use label_list::{LabelEntry, LabelList, LabelListError};
pub use prop::{Prop, PropId, PropKind};
pub use provider::external::{ExternalProvider, ExternalProviderId};
pub use provider::internal::{InternalProvider, InternalProviderId};
pub use provider::ProviderArity;
pub use provider::ProviderKind;
pub use schema::variant::root_prop::component_type::ComponentType;
pub use schema::{Schema, SchemaError, SchemaId, SchemaVariant, SchemaVariantId};
pub use secret::Secret;
pub use secret::SecretError;
pub use secret::SecretId;
pub use secret::SecretView;
pub use secret::{EncryptedSecret, SecretAlgorithm, SecretVersion};
pub use standard_model::{StandardModel, StandardModelError, StandardModelResult};
pub use tenancy::{Tenancy, TenancyError};
pub use timestamp::{Timestamp, TimestampError};
pub use user::{User, UserClaim, UserError, UserPk, UserResult};
pub use visibility::{Visibility, VisibilityError};
pub use workspace::{Workspace, WorkspaceError, WorkspacePk, WorkspaceResult};
pub use workspace_snapshot::graph::WorkspaceSnapshotGraph;
pub use workspace_snapshot::WorkspaceSnapshot;
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
    ContentStorePg(#[from] content_store::StoreError),
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
    migrate(
        services_context.pg_pool(),
        services_context.content_store_pg_pool(),
    )
    .await?;
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
pub async fn migrate(pg: &PgPool, content_store_pg_pool: &PgPool) -> ModelResult<()> {
    content_store::PgStore::migrate(content_store_pg_pool).await?;
    pg.migrate(embedded::migrations::runner()).await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[instrument(level = "info", skip_all)]
pub async fn migrate_local_builtins(
    dal_pg: &PgPool,
    nats: &NatsClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    veritech: veritech_client::Client,
    encryption_key: &CycloneEncryptionKey,
    selected_test_builtin_schemas: Option<SelectedTestBuiltinSchemas>,
    pkgs_path: PathBuf,
    module_index_url: String,
    symmetric_crypto_service: &SymmetricCryptoService,
    rebaser_config: RebaserClientConfig,
    content_store_pg_pool: &PgPool,
) -> ModelResult<()> {
    let services_context = ServicesContext::new(
        dal_pg.clone(),
        nats.clone(),
        job_processor,
        veritech,
        Arc::new(*encryption_key),
        Some(pkgs_path),
        Some(module_index_url),
        symmetric_crypto_service.clone(),
        rebaser_config,
        content_store_pg_pool.clone(),
    );
    let dal_context = services_context.into_builder(true);
    let mut ctx = dal_context.build_default().await?;

    let workspace = Workspace::builtin(&mut ctx).await?;
    ctx.update_tenancy(Tenancy::new(*workspace.pk()));
    ctx.update_to_head();
    ctx.update_snapshot_to_visibility().await?;

    builtins::migrate_local(&ctx, selected_test_builtin_schemas).await?;

    ctx.blocking_commit().await?;

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
    EnumVariantNames,
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

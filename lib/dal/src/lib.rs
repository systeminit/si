//! The Data Access Layer (DAL) for System Initiative.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use si_data_nats::{NatsClient, NatsError};
use si_data_pg::{PgError, PgPool, PgPoolError};
use strum::{Display, EnumString, EnumVariantNames};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time;
use tokio::time::Instant;
use veritech_client::{Client, EncryptionKey};

use crate::builtins::SelectedTestBuiltinSchemas;

pub mod action;
pub mod action_prototype;
pub mod actor_view;
pub mod attribute;
pub mod builtins;
pub mod change_set;
pub mod change_status;
pub mod code_view;
pub mod component;
pub mod content;
pub mod context;
pub mod cyclone_key_pair;
pub mod diagram;
pub mod edge;
pub mod fix;
pub mod func;
pub mod history_event;
pub mod index_map;
pub mod installed_pkg;
pub mod job;
pub mod job_failure;
pub mod jwt_key;
pub mod key_pair;
pub mod label_list;
pub mod node;
pub mod node_menu;
pub mod pkg;
pub mod prop;
pub mod prop_tree;
pub mod property_editor;
pub mod prototype_context;
pub mod prototype_list_for_func;
pub mod provider;
pub mod qualification;
pub mod reconciliation_prototype;
pub mod schema;
pub mod secret;
pub mod socket;
pub mod standard_accessors;
pub mod standard_model;
pub mod standard_pk;
pub mod status;
pub mod tasks;
pub mod tenancy;
pub mod timestamp;
pub mod user;
pub mod validation;
pub mod visibility;
pub mod workspace;
pub mod workspace_snapshot;
pub mod ws_event;

pub use action::{Action, ActionError, ActionId};
pub use action_prototype::{
    ActionKind, ActionPrototype, ActionPrototypeContext, ActionPrototypeError, ActionPrototypeId,
    ActionPrototypeView,
};
pub use actor_view::ActorView;
pub use attribute::value::view::AttributeView;
pub use attribute::{
    context::{
        AttributeContext, AttributeContextBuilder, AttributeContextBuilderError,
        AttributeContextError, AttributeReadContext,
    },
    prototype::argument::{
        AttributePrototypeArgument, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
        AttributePrototypeArgumentResult,
    },
    prototype::{
        AttributePrototype, AttributePrototypeError, AttributePrototypeId, AttributePrototypeResult,
    },
    value::{
        AttributeValue, AttributeValueError, AttributeValueId, AttributeValuePayload,
        AttributeValueResult,
    },
};
pub use builtins::{BuiltinsError, BuiltinsResult};
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetPk, ChangeSetStatus};
pub use code_view::{CodeLanguage, CodeView};
pub use component::{
    resource::ResourceView, status::ComponentStatus, status::HistoryActorTimestamp, Component,
    ComponentError, ComponentId, ComponentView, ComponentViewProperties,
};
use content::hash::ContentHash;
pub use context::{
    AccessBuilder, Connections, DalContext, DalContextBuilder, RequestContext, ServicesContext,
    Transactions, TransactionsError,
};
pub use cyclone_key_pair::CycloneKeyPair;
pub use diagram::{
    connection::Connection, connection::DiagramEdgeView, Diagram, DiagramError, DiagramKind,
};
pub use edge::{Edge, EdgeError, EdgeResult};
pub use fix::batch::{FixBatch, FixBatchId};
pub use fix::resolver::{FixResolver, FixResolverError, FixResolverId};
pub use fix::{Fix, FixCompletionStatus, FixError, FixId};
pub use func::argument::FuncArgument;
pub use func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError};
pub use func::{
    backend::{FuncBackendError, FuncBackendKind, FuncBackendResponseType},
    binding::{FuncBinding, FuncBindingError, FuncBindingId},
    Func, FuncError, FuncId, FuncResult,
};
pub use history_event::{HistoryActor, HistoryEvent, HistoryEventError};
pub use index_map::IndexMap;
pub use job::definition::DependentValuesUpdate;
pub use job::processor::{JobQueueProcessor, NatsProcessor};
pub use job_failure::{JobFailure, JobFailureError, JobFailureResult};
pub use jwt_key::JwtPublicSigningKey;
pub use key_pair::{KeyPair, KeyPairError, KeyPairResult, PublicKey};
pub use label_list::{LabelEntry, LabelList, LabelListError};
pub use node::NodeId;
pub use node::{Node, NodeError, NodeKind};
pub use node_menu::NodeMenuError;
pub use prop::{Prop, PropError, PropId, PropKind, PropPk, PropResult};
pub use prototype_context::HasPrototypeContext;
pub use prototype_list_for_func::{
    PrototypeListForFunc, PrototypeListForFuncError, PrototypeListForFuncResult,
};
pub use provider::external::{ExternalProvider, ExternalProviderError, ExternalProviderId};
pub use provider::internal::{InternalProvider, InternalProviderError, InternalProviderId};
pub use qualification::{QualificationError, QualificationView};
pub use reconciliation_prototype::{
    ReconciliationPrototype, ReconciliationPrototypeContext, ReconciliationPrototypeError,
    ReconciliationPrototypeId,
};
pub use schema::variant::leaves::LeafInput;
pub use schema::variant::leaves::LeafInputLocation;
pub use schema::variant::leaves::LeafKind;
pub use schema::variant::root_prop::component_type::ComponentType;
pub use schema::variant::root_prop::RootProp;
pub use schema::variant::root_prop::RootPropChild;
pub use schema::variant::SchemaVariantError;
pub use schema::{Schema, SchemaError, SchemaId, SchemaPk, SchemaVariant, SchemaVariantId};
pub use secret::{
    DecryptedSecret, EncryptedSecret, Secret, SecretAlgorithm, SecretError, SecretId, SecretKind,
    SecretObjectType, SecretPk, SecretResult, SecretVersion,
};
pub use socket::{Socket, SocketArity, SocketId};
pub use standard_model::{StandardModel, StandardModelError, StandardModelResult};
pub use status::{
    StatusUpdate, StatusUpdateError, StatusUpdateResult, StatusUpdater, StatusUpdaterError,
};
pub use tenancy::{Tenancy, TenancyError};
pub use timestamp::{Timestamp, TimestampError};
pub use user::{User, UserClaim, UserError, UserPk, UserResult};
pub use validation::prototype::{
    context::ValidationPrototypeContext, ValidationPrototype, ValidationPrototypeError,
    ValidationPrototypeId,
};
pub use validation::resolver::{
    ValidationResolver, ValidationResolverError, ValidationResolverId, ValidationStatus,
};
pub use visibility::{Visibility, VisibilityError};
pub use workspace::{Workspace, WorkspaceError, WorkspacePk, WorkspaceResult, WorkspaceSignup};
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
    Migration(#[from] PgPoolError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error("database error")]
    PgError(#[from] PgError),
    #[error("transactions error")]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}

pub type ModelResult<T> = Result<T, ModelError>;

#[instrument(skip_all)]
pub async fn migrate_all(
    pg: &PgPool,
    nats: &NatsClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    veritech: Client,
    encryption_key: &EncryptionKey,
    pkgs_path: PathBuf,
    module_index_url: String,
) -> ModelResult<()> {
    migrate(pg).await?;
    migrate_builtins(
        pg,
        nats,
        job_processor,
        veritech,
        encryption_key,
        None,
        pkgs_path,
        module_index_url,
    )
    .await?;
    Ok(())
}

#[instrument(skip_all)]
pub async fn migrate_all_with_progress(
    pg: &PgPool,
    nats: &NatsClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    veritech: Client,
    encryption_key: &EncryptionKey,
    pkgs_path: PathBuf,
    module_index_url: String,
) -> ModelResult<()> {
    let mut interval = time::interval(Duration::from_secs(5));
    let instant = Instant::now();
    let migrate_all = migrate_all(
        pg,
        nats,
        job_processor,
        veritech,
        encryption_key,
        pkgs_path,
        module_index_url,
    );
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

#[instrument(skip_all)]
pub async fn migrate(pg: &PgPool) -> ModelResult<()> {
    Ok(pg.migrate(embedded::migrations::runner()).await?)
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip_all)]
pub async fn migrate_builtins(
    pg: &PgPool,
    nats: &NatsClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    veritech: veritech_client::Client,
    encryption_key: &EncryptionKey,
    selected_test_builtin_schemas: Option<SelectedTestBuiltinSchemas>,
    pkgs_path: PathBuf,
    module_index_url: String,
) -> ModelResult<()> {
    let services_context = ServicesContext::new(
        pg.clone(),
        nats.clone(),
        job_processor,
        veritech,
        Arc::new(*encryption_key),
        Some(pkgs_path),
        Some(module_index_url),
    );
    let dal_context = services_context.into_builder(true);
    let mut ctx = dal_context.build_default().await?;

    let workspace = Workspace::builtin(&ctx).await?;
    ctx.update_tenancy(Tenancy::new(*workspace.pk()));
    ctx.blocking_commit().await?;

    builtins::migrate(&ctx, selected_test_builtin_schemas).await?;

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
    use super::*;
    use serde::{Deserialize, Serialize};

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

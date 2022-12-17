//! The Data Access Layer (DAL) for System Initiative.

use std::io;
use std::sync::Arc;

use rand::Rng;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use si_data_nats::{NatsClient, NatsError};
use si_data_pg::{PgError, PgPool, PgPoolError};
use strum_macros::{Display, EnumString, EnumVariantNames};
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::{Client, EncryptionKey};

pub mod action_prototype;
pub mod actor_view;
pub mod attribute;
pub mod billing_account;
pub mod builtins;
pub mod capability;
pub mod change_set;
pub mod code_view;
pub mod component;
pub mod confirmation_prototype;
pub mod confirmation_resolver;
pub mod confirmation_resolver_tree;
pub mod confirmation_status;
pub mod context;
pub mod cyclone_key_pair;
pub mod diagram;
pub mod edge;
pub mod edit_field;
pub mod fix;
pub mod func;
pub mod group;
pub mod history_event;
pub mod index_map;
pub mod job;
pub mod job_failure;
pub mod jwt_key;
pub mod key_pair;
pub mod label_list;
pub mod node;
pub mod node_menu;
pub mod node_position;
pub mod organization;
pub mod prop;
pub mod prop_tree;
pub mod property_editor;
pub mod prototype_context;
pub mod prototype_list_for_func;
pub mod provider;
pub mod qualification;
pub mod read_tenancy;
pub mod resource_scheduler;
pub mod schema;
pub mod secret;
pub mod socket;
pub mod standard_accessors;
pub mod standard_model;
pub mod standard_pk;
pub mod status;
pub mod timestamp;
pub mod user;
pub mod validation;
pub mod visibility;
pub mod workflow;
pub mod workflow_prototype;
pub mod workflow_resolver;
pub mod workflow_runner;
pub mod workspace;
pub mod write_tenancy;
pub mod ws_event;

pub use action_prototype::{
    ActionPrototype, ActionPrototypeContext, ActionPrototypeError, ActionPrototypeId,
};
pub use actor_view::ActorView;
pub use attribute::value::view::AttributeView;
pub use attribute::{
    context::{
        AttributeContext, AttributeContextBuilderError, AttributeContextError, AttributeReadContext,
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
pub use billing_account::{
    BillingAccount, BillingAccountDefaults, BillingAccountError, BillingAccountId,
    BillingAccountPk, BillingAccountSignup,
};
pub use builtins::{BuiltinsError, BuiltinsResult};
pub use capability::{Capability, CapabilityError, CapabilityId, CapabilityPk, CapabilityResult};
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetPk, ChangeSetStatus};
pub use code_view::{CodeLanguage, CodeView};
pub use component::{
    resource::ResourceView, Component, ComponentError, ComponentId, ComponentStatus, ComponentView,
    HistoryActorTimestamp,
};
pub use confirmation_prototype::{
    ConfirmationPrototype, ConfirmationPrototypeContext, ConfirmationPrototypeError,
    ConfirmationPrototypeId,
};
pub use confirmation_resolver::{
    ConfirmationResolver, ConfirmationResolverContext, ConfirmationResolverError,
    ConfirmationResolverId,
};
pub use confirmation_resolver_tree::{ConfirmationResolverTree, ConfirmationResolverTreeError};
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
pub use fix::resolver::{FixResolver, FixResolverContext, FixResolverError, FixResolverId};
pub use fix::{Fix, FixCompletionStatus, FixId};
pub use func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError};
pub use func::{
    backend::{FuncBackendError, FuncBackendKind, FuncBackendResponseType},
    binding::{FuncBinding, FuncBindingError, FuncBindingId},
    Func, FuncError, FuncId, FuncResult,
};
pub use group::{Group, GroupError, GroupId, GroupResult};
pub use history_event::{HistoryActor, HistoryEvent, HistoryEventError};
pub use index_map::IndexMap;
pub use job::processor::{faktory_processor::FaktoryProcessor, JobQueueProcessor};
pub use job_failure::{JobFailure, JobFailureError, JobFailureResult};
pub use jwt_key::{create_jwt_key_if_missing, JwtSecretKey};
pub use key_pair::{KeyPair, KeyPairError, KeyPairResult, PublicKey};
pub use label_list::{LabelEntry, LabelList, LabelListError};
pub use node::{Node, NodeError, NodeKind, NodeTemplate, NodeView};
pub use node_menu::NodeMenuError;
pub use node_position::{
    NodePosition, NodePositionError, NodePositionId, NodePositionPk, NodePositionResult,
};
pub use organization::{
    Organization, OrganizationError, OrganizationId, OrganizationPk, OrganizationResult,
};
pub use prop::{Prop, PropError, PropId, PropKind, PropPk, PropResult};
pub use prototype_context::HasPrototypeContext;
pub use prototype_list_for_func::{
    PrototypeListForFunc, PrototypeListForFuncError, PrototypeListForFuncResult,
};
pub use provider::external::{ExternalProvider, ExternalProviderError, ExternalProviderId};
pub use provider::internal::{InternalProvider, InternalProviderError, InternalProviderId};
pub use qualification::{QualificationError, QualificationView};
pub use read_tenancy::{ReadTenancy, ReadTenancyError};
pub use resource_scheduler::{ResourceScheduler, ResourceSchedulerError};
pub use schema::variant::root_prop::RootProp;
pub use schema::{
    Schema, SchemaError, SchemaId, SchemaKind, SchemaPk, SchemaVariant, SchemaVariantId,
};
pub use secret::{
    DecryptedSecret, EncryptedSecret, Secret, SecretAlgorithm, SecretError, SecretId, SecretKind,
    SecretObjectType, SecretPk, SecretResult, SecretVersion,
};
pub use socket::{Socket, SocketArity, SocketId};
pub use standard_model::{StandardModel, StandardModelError, StandardModelResult};
pub use status::{
    StatusUpdate, StatusUpdateError, StatusUpdateResult, StatusUpdater, StatusUpdaterError,
};
pub use timestamp::{Timestamp, TimestampError};
pub use user::{User, UserClaim, UserError, UserId, UserPk, UserResult};
pub use validation::prototype::{
    context::ValidationPrototypeContext, ValidationPrototype, ValidationPrototypeError,
    ValidationPrototypeId,
};
pub use validation::resolver::{
    ValidationResolver, ValidationResolverError, ValidationResolverId, ValidationStatus,
};
pub use visibility::{Visibility, VisibilityError};
pub use workflow::{
    WorkflowError, WorkflowKind, WorkflowResult, WorkflowStep, WorkflowTree, WorkflowTreeStep,
    WorkflowView,
};
pub use workflow_prototype::{
    WorkflowPrototype, WorkflowPrototypeContext, WorkflowPrototypeError, WorkflowPrototypeId,
};
pub use workflow_resolver::{WorkflowResolver, WorkflowResolverError, WorkflowResolverId};
pub use workflow_runner::workflow_runner_state::{
    WorkflowRunnerState, WorkflowRunnerStateId, WorkflowRunnerStatus,
};
pub use workflow_runner::{WorkflowRunner, WorkflowRunnerError, WorkflowRunnerId};
pub use workspace::{Workspace, WorkspaceError, WorkspaceId, WorkspacePk, WorkspaceResult};
pub use write_tenancy::{WriteTenancy, WriteTenancyError};
pub use ws_event::{WsEvent, WsEventError, WsEventResult, WsPayload};

#[cfg(debug_assertions)]
const REQUIRED_BINARIES: &str = include_str!("../../../scripts/data/required-binaries.txt");
#[cfg(debug_assertions)]
const REQUIRED_COMMANDS: &str = include_str!("../../../scripts/data/required-commands.txt");

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("command not found from parts: {0:?}")]
    CommandNotFoundFromParts(Vec<&'static str>),
    #[error(
        "missing required runtime dependencies ({0:?}) and/or required commands failed ({1:?})"
    )]
    MissingRequiredRuntimeDependencies(Vec<&'static str>, Vec<(&'static str, io::Error)>),
    #[error("failed to initialize sodium oxide")]
    SodiumOxide,
}

pub type InitializationResult<T> = Result<T, InitializationError>;

/// Perform base initializations before using the `dal`.
pub fn init() -> InitializationResult<()> {
    sodiumoxide::init().map_err(|()| InitializationError::SodiumOxide)?;

    #[cfg(debug_assertions)]
    check_runtime_dependencies()?;

    Ok(())
}

/// Check that runtime dependencies are in `PATH`. This check only occurs in "dev mode" during
/// [`init()`](init()).
#[cfg(debug_assertions)]
pub fn check_runtime_dependencies() -> InitializationResult<()> {
    use std::collections::VecDeque;
    use std::process::{Command, Stdio};

    let mut missing_binaries = Vec::new();
    let mut failed_commands = Vec::new();

    for binary in REQUIRED_BINARIES.lines() {
        if which::which(binary).is_err() {
            missing_binaries.push(binary);
        }
    }
    for command in REQUIRED_COMMANDS.lines() {
        let mut parts = command.split(' ').collect::<VecDeque<&str>>();
        let command = parts.pop_front().ok_or_else(|| {
            InitializationError::CommandNotFoundFromParts(Vec::from(parts.clone()))
        })?;
        if let Err(e) = Command::new(command)
            .args(parts)
            .stdout(Stdio::null())
            .spawn()
        {
            failed_commands.push((command, e))
        }
    }

    if !missing_binaries.is_empty() {
        return Err(InitializationError::MissingRequiredRuntimeDependencies(
            missing_binaries,
            failed_commands,
        ));
    }
    Ok(())
}

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}

const NAME_CHARSET: &[u8] = b"0123456789";

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
}

pub type ModelResult<T> = Result<T, ModelError>;

#[instrument(skip_all)]
pub async fn migrate_all(
    pg: &PgPool,
    nats: &NatsClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    veritech: Client,
    encryption_key: &EncryptionKey,
) -> ModelResult<()> {
    migrate(pg).await?;
    migrate_builtins(pg, nats, job_processor, veritech, encryption_key, false).await?;
    Ok(())
}

#[instrument(skip_all)]
pub async fn migrate(pg: &PgPool) -> ModelResult<()> {
    Ok(pg.migrate(embedded::migrations::runner()).await?)
}

#[instrument(skip_all)]
pub async fn migrate_builtins(
    pg: &PgPool,
    nats: &NatsClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    veritech: veritech_client::Client,
    encryption_key: &EncryptionKey,
    skip_migrating_schemas: bool,
) -> ModelResult<()> {
    let services_context = ServicesContext::new(
        pg.clone(),
        nats.clone(),
        job_processor,
        veritech,
        Arc::new(*encryption_key),
    );
    let dal_context = services_context.into_builder();
    let request_context = RequestContext::new_universal_head(HistoryActor::SystemInit);
    let ctx = dal_context.build(request_context).await?;
    builtins::migrate(&ctx, skip_migrating_schemas).await?;
    ctx.commit().await?;
    Ok(())
}

pub fn generate_name() -> String {
    let mut rng = rand::thread_rng();
    let unique_id: String = (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..NAME_CHARSET.len());
            NAME_CHARSET[idx] as char
        })
        .collect();
    format!("si-{}", unique_id)
}

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

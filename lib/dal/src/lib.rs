//! The Data Access Layer (DAL) for System Initiative.

use std::sync::Arc;

use rand::Rng;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use si_data::{NatsClient, NatsError, PgError, PgPool, PgPoolError};
use strum_macros::{Display, EnumString, EnumVariantNames};
use telemetry::prelude::*;
use thiserror::Error;

pub mod attribute;
pub mod billing_account;
pub mod builtins;
pub mod capability;
pub mod change_set;
pub mod code_generation_prototype;
pub mod code_generation_resolver;
pub mod code_view;
pub mod component;
pub mod context;
pub mod cyclone_key_pair;
pub mod diagram;
pub mod edge;
pub mod edit_field;
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
pub mod property_editor;
pub mod provider;
pub mod qualification;
pub mod qualification_check;
pub mod qualification_prototype;
pub mod qualification_resolver;
pub mod read_tenancy;
pub mod resource;
pub mod resource_prototype;
pub mod resource_resolver;
pub mod resource_scheduler;
pub mod schema;
pub mod secret;
pub mod socket;
pub mod standard_accessors;
pub mod standard_model;
pub mod standard_pk;
pub mod system;
pub mod test;
pub mod test_harness;
pub mod timestamp;
pub mod user;
pub mod validation_prototype;
pub mod validation_resolver;
pub mod visibility;
pub mod workflow;
pub mod workflow_prototype;
pub mod workflow_resolver;
pub mod workflow_runner;
pub mod workspace;
pub mod write_tenancy;
pub mod ws_event;

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
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetPk, ChangeSetStatus, NO_CHANGE_SET_PK};
pub use code_generation_prototype::{
    CodeGenerationPrototype, CodeGenerationPrototypeError, CodeGenerationPrototypeId,
};
pub use code_generation_resolver::{
    CodeGenerationResolver, CodeGenerationResolverError, CodeGenerationResolverId,
};
pub use code_view::{CodeLanguage, CodeView};
pub use component::{Component, ComponentError, ComponentId, ComponentView};
pub use context::{
    AccessBuilder, Connections, DalContext, DalContextBuilder, RequestContext, ServicesContext,
    Transactions, TransactionsError,
};
pub use cyclone_key_pair::CycloneKeyPair;
pub use diagram::{
    connection::Connection, connection::DiagramEdgeView, Diagram, DiagramError, DiagramKind,
};
pub use edge::{Edge, EdgeError, EdgeResult};
pub use func::binding_return_value::FuncBindingReturnValue;
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
pub use provider::external::{ExternalProvider, ExternalProviderError, ExternalProviderId};
pub use provider::internal::{InternalProvider, InternalProviderError, InternalProviderId};
pub use qualification::{QualificationError, QualificationView};
pub use qualification_check::{
    QualificationCheck, QualificationCheckError, QualificationCheckId, QualificationCheckPk,
};
pub use qualification_prototype::{
    QualificationPrototype, QualificationPrototypeError, QualificationPrototypeId,
};
pub use qualification_resolver::{
    QualificationResolver, QualificationResolverError, QualificationResolverId,
};
pub use read_tenancy::{ReadTenancy, ReadTenancyError};
pub use resource::{Resource, ResourceError, ResourceView};
pub use resource_prototype::{ResourcePrototype, ResourcePrototypeError, ResourcePrototypeId};
pub use resource_resolver::{ResourceResolver, ResourceResolverError, ResourceResolverId};
pub use resource_scheduler::{ResourceScheduler, ResourceSchedulerError};
pub use schema::{
    Schema, SchemaError, SchemaId, SchemaKind, SchemaPk, SchemaVariant, SchemaVariantId,
};
pub use secret::{
    DecryptedSecret, EncryptedSecret, Secret, SecretAlgorithm, SecretError, SecretId, SecretKind,
    SecretObjectType, SecretPk, SecretResult, SecretVersion,
};
pub use socket::{Socket, SocketId};
pub use standard_model::{StandardModel, StandardModelError, StandardModelResult};
pub use system::{System, SystemError, SystemId, SystemPk, SystemResult};
pub use timestamp::{Timestamp, TimestampError};
pub use user::{User, UserClaim, UserError, UserId, UserResult};
pub use validation_prototype::{
    ValidationPrototype, ValidationPrototypeError, ValidationPrototypeId,
};
pub use validation_resolver::{ValidationResolver, ValidationResolverError, ValidationResolverId};
use veritech::EncryptionKey;
pub use visibility::{Visibility, VisibilityError};
pub use workflow::{
    WorkflowError, WorkflowKind, WorkflowResult, WorkflowStep, WorkflowTree, WorkflowTreeStep,
    WorkflowView,
};
pub use workflow_prototype::{
    WorkflowPrototype, WorkflowPrototypeContext, WorkflowPrototypeError, WorkflowPrototypeId,
};
pub use workflow_resolver::{WorkflowResolver, WorkflowResolverError, WorkflowResolverId};
pub use workflow_runner::{WorkflowRunner, WorkflowRunnerError, WorkflowRunnerId};
pub use workspace::{Workspace, WorkspaceError, WorkspaceId, WorkspacePk, WorkspaceResult};
pub use write_tenancy::{WriteTenancy, WriteTenancyError};
pub use ws_event::{WsEvent, WsEventError, WsPayload};

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("failed ot initialize sodium oxide")]
    SodiumOxide,
}

pub fn init() -> Result<(), InitializationError> {
    sodiumoxide::init().map_err(|()| InitializationError::SodiumOxide)
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
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> ModelResult<()> {
    migrate(pg).await?;
    migrate_builtins(pg, nats, job_processor, veritech, encryption_key).await?;
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
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
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
    builtins::migrate(&ctx).await?;
    ctx.commit().await?;
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

//! The Data Access Layer (DAL) for System Initiative.

use rand::Rng;
use si_data::{NatsClient, NatsError, PgError, PgPool, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;

pub mod attribute_prototype;
pub mod attribute_resolver;
pub mod attribute_resolver_context;
pub mod billing_account;
pub mod capability;
pub mod change_set;
pub mod code_generation_prototype;
pub mod code_generation_resolver;
pub mod code_view;
pub mod component;
pub mod cyclone_public_key;
pub mod edge;
pub mod edit_field;
pub mod edit_session;
pub mod func;
pub mod group;
pub mod history_event;
pub mod index_map;
pub mod jwt_key;
pub mod key_pair;
pub mod label_list;
pub mod node;
pub mod node_menu;
pub mod node_position;
pub mod organization;
pub mod prop;
pub mod qualification;
pub mod qualification_check;
pub mod qualification_prototype;
pub mod qualification_resolver;
pub mod resource;
pub mod resource_prototype;
pub mod resource_resolver;
pub mod resource_scheduler;
pub mod schema;
pub mod schematic;
pub mod secret;
pub mod socket;
pub mod standard_accessors;
pub mod standard_model;
pub mod standard_pk;
pub mod system;
pub mod tenancy;
pub mod test_harness;
pub mod timestamp;
pub mod user;
pub mod validation_prototype;
pub mod validation_resolver;
pub mod visibility;
pub mod workspace;
pub mod ws_event;

pub use attribute_prototype::{AttributePrototype, AttributePrototypeError, AttributePrototypeId};
pub use attribute_resolver::{
    AttributeResolver, AttributeResolverError, AttributeResolverId, AttributeResolverValue,
};
pub use billing_account::{
    BillingAccount, BillingAccountDefaults, BillingAccountError, BillingAccountId, BillingAccountPk,
};
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
pub use cyclone_public_key::CyclonePublicKey;
pub use edge::{Edge, EdgeError, EdgeResult};
pub use edit_session::{
    EditSession, EditSessionError, EditSessionPk, EditSessionStatus, NO_EDIT_SESSION_PK,
};
pub use func::{
    backend::{FuncBackendError, FuncBackendKind, FuncBackendResponseType},
    Func, FuncError, FuncResult,
};
pub use group::{Group, GroupError, GroupId, GroupResult};
pub use history_event::{HistoryActor, HistoryEvent, HistoryEventError};
pub use index_map::IndexMap;
pub use jwt_key::{create_jwt_key_if_missing, JwtSecretKey};
pub use key_pair::{KeyPair, KeyPairError, KeyPairResult, PublicKey};
pub use label_list::{LabelEntry, LabelList, LabelListError};
pub use node::{Node, NodeError, NodeKind, NodeTemplate, NodeView};
pub use node_menu::{MenuFilter, NodeMenuError};
pub use node_position::{
    NodePosition, NodePositionError, NodePositionId, NodePositionPk, NodePositionResult,
};
pub use organization::{
    Organization, OrganizationError, OrganizationId, OrganizationPk, OrganizationResult,
};
pub use prop::{Prop, PropError, PropId, PropKind, PropPk, PropResult};
pub use qualification_check::{
    QualificationCheck, QualificationCheckError, QualificationCheckId, QualificationCheckPk,
};
pub use qualification_prototype::{
    QualificationPrototype, QualificationPrototypeError, QualificationPrototypeId,
};
pub use qualification_resolver::{
    QualificationResolver, QualificationResolverError, QualificationResolverId,
};
pub use resource::{Resource, ResourceError, ResourceView};
pub use resource_prototype::{ResourcePrototype, ResourcePrototypeError, ResourcePrototypeId};
pub use resource_resolver::{ResourceResolver, ResourceResolverError, ResourceResolverId};
pub use resource_scheduler::{ResourceScheduler, ResourceSchedulerError};
pub use schema::{
    Schema, SchemaError, SchemaId, SchemaKind, SchemaPk, SchemaVariant, SchemaVariantId,
};
pub use schematic::{Connection, Schematic, SchematicError, SchematicKind};
pub use secret::{
    DecryptedSecret, EncryptedSecret, Secret, SecretAlgorithm, SecretError, SecretId, SecretKind,
    SecretObjectType, SecretPk, SecretResult, SecretVersion,
};
pub use standard_model::{StandardModel, StandardModelError, StandardModelResult};
pub use system::{System, SystemError, SystemId, SystemPk, SystemResult};
pub use tenancy::{Tenancy, TenancyError};
pub use timestamp::{Timestamp, TimestampError};
pub use user::{User, UserClaim, UserError, UserId, UserResult};
pub use validation_prototype::{
    ValidationPrototype, ValidationPrototypeError, ValidationPrototypeId,
};
pub use validation_resolver::{ValidationResolver, ValidationResolverError, ValidationResolverId};
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
    #[error("Func error")]
    Func(#[from] FuncError),
}

pub type ModelResult<T> = Result<T, ModelError>;

#[instrument(skip_all)]
pub async fn migrate(pg: &PgPool) -> ModelResult<()> {
    let result = pg.migrate(embedded::migrations::runner()).await?;
    Ok(result)
}

#[instrument(skip_all)]
pub async fn migrate_builtin_schemas(
    pg: &PgPool,
    nats: &NatsClient,
    veritech: veritech::Client,
) -> ModelResult<()> {
    let mut conn = pg.get().await?;
    let txn = conn.transaction().await?;
    let nats = nats.transaction();
    func::builtins::migrate(&txn, &nats).await?;
    schema::builtins::migrate(&txn, &nats, veritech).await?;
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

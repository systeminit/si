use rand::Rng;
use thiserror::Error;
use tracing::instrument;

pub use action::{Action, ActionError, ActionResult};
pub use api_client::{ApiClaim, ApiClient, ApiClientError, ApiClientKind, ApiClientResult};
pub use application::{
    ApplicationContext, ApplicationEntities, ApplicationError, ApplicationListEntry,
    ApplicationResult,
};
pub use billing_account::{BillingAccount, BillingAccountError, BillingAccountResult};
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetResult, ChangeSetStatus, SiChangeSet};
pub use connection::{Connection, ConnectionError, ConnectionPoint, Connections};
pub use discovery::{DiscoveryError, DiscoveryListEntry};
pub use edge::{Edge, EdgeError, EdgeKind, EdgeResult, Edges, Vertex};
pub use edit_session::{EditSession, EditSessionError, EditSessionResult};
pub use entity::diff::{diff_for_props, DiffError};
pub use entity::{Entity, EntityError};
pub use event::{Event, EventError, EventKind, EventResult, EventStatus};
pub use event_log::{EventLog, EventLogError, EventLogLevel, EventLogResult};
pub use group::{Capability, Group, GroupError, GroupResult};
pub use jwt_key::{
    create_jwt_key_if_missing, get_jwt_signing_key, get_jwt_validation_key, validate_bearer_token,
    validate_bearer_token_api_client, JwtKeyError, JwtKeyResult,
};
pub use key_pair::{KeyPair, KeyPairError, PublicKey};
pub use label_list::{LabelList, LabelListItem};
pub use node::{Node, NodeError};
pub use node_position::{NodePosition, NodePositionError};
pub use organization::{Organization, OrganizationError};
pub use output_line::{OutputLine, OutputLineStream};
pub use prop::{Prop, PropError, PropKind};
pub use prop_variant::{PropVariant, PropVariantError};
pub use qualification::{Qualification, QualificationError};
pub use remote_function::{RemoteFunctionRequest, RemoteFunctionResult};
pub use resolver::{
    Resolver, ResolverArgKindBinding, ResolverBackendKind, ResolverBackendKindArrayBinding,
    ResolverBackendKindBinding, ResolverBackendKindNumberBinding, ResolverBackendKindObjectBinding,
    ResolverBackendKindStringBinding, ResolverBinding, ResolverError, ResolverOutputKind,
};
pub use resource::{
    Resource, ResourceError, ResourceInternalHealth, ResourceInternalStatus, ResourceResult,
};
pub use schema::{Schema, SchemaError, SchemaMap, SchemaResult};
pub use schema_variant::{SchemaVariant, SchemaVariantError, SchemaVariantResult};
pub use schematic::{Schematic, SchematicError, SchematicKind, SchematicNode, SchematicResult};
pub use secret::{
    EncryptedSecret, Secret, SecretAlgorithm, SecretError, SecretKind, SecretObjectType,
    SecretResult, SecretVersion,
};
pub use session::{SessionError, SessionResult};
use si_data::{NatsConn, NatsTxnError, PgError, PgPool, PgPoolError};
pub use si_storable::{MinimalStorable, SiStorable, SimpleStorable};
pub use support::lodash::{self, LodashError};
pub use support::veritech::{Veritech, VeritechError};
pub use system::{SystemError, SystemResult};
pub use user::{LoginReply, LoginRequest, SiClaims, User, UserError, UserResult};
pub use visualization::{
    ActivitySummary, ChangesSummary, ResourceSummary, VisualizationError, VisualizationResult,
};
pub use workflow::{Workflow, WorkflowContext, WorkflowError, WorkflowRun};
pub use workspace::{Workspace, WorkspaceError};

pub static mut PAGE_SECRET_KEY: Option<sodiumoxide::crypto::secretbox::Key> = None;

pub fn page_secret_key() -> &'static sodiumoxide::crypto::secretbox::Key {
    unsafe {
        PAGE_SECRET_KEY
            .as_ref()
            .expect("cannot unwrap page secret key - it should be set before you call this!")
    }
}

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}

pub mod action;
pub mod api_client;
pub mod application;
pub mod billing_account;
pub mod change_set;
pub mod connection;
pub mod discovery;
pub mod edge;
pub mod edit_session;
pub mod entity;
pub mod event;
pub mod event_log;
pub mod group;
pub mod jwt_key;
pub mod key_pair;
pub mod label_list;
pub mod node;
pub mod node_position;
pub mod organization;
pub mod output_line;
pub mod prop;
pub mod prop_variant;
pub mod qualification;
pub mod remote_function;
pub mod resolver;
pub mod resource;
pub mod schema;
pub mod schema_builtins;
pub mod schema_variant;
pub mod schematic;
pub mod secret;
pub mod session;
pub mod si_storable;
pub mod support;
pub mod system;
pub mod test;
pub mod user;
pub mod visualization;
pub mod workflow;
pub mod workspace;

const NAME_CHARSET: &[u8] = b"0123456789";

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("migration error: {0}")]
    Migration(#[from] PgPoolError),
    #[error("schema builtin error: {0}")]
    SchemaBuiltin(#[from] SchemaError),
    #[error("database error: {0}")]
    PgError(#[from] PgError),
    #[error("nats txn error: {0}")]
    NatsTxnError(#[from] NatsTxnError),
}
pub type ModelResult<T> = Result<T, ModelError>;

#[instrument(skip(pg))]
pub async fn migrate(pg: &PgPool) -> ModelResult<()> {
    let result = pg.migrate(embedded::migrations::runner()).await?;
    Ok(result)
}

#[instrument(skip(pg, nats))]
pub async fn migrate_builtin_schemas(pg: &PgPool, nats: &NatsConn) -> ModelResult<()> {
    let mut conn = pg.get().await?;
    let txn = conn.transaction().await?;
    let nats = nats.transaction();
    schema_builtins::system::migrate(&txn, &nats).await?;
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
    //let mut name_generator = names::Generator::with_naming(names::Name::Numbered);
    //let name = name_generator.next().unwrap();
    //return name;
}

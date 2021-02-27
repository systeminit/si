use thiserror::Error;

use si_data::{PgError, PgPool};

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

pub mod api_client;
pub mod application;
pub mod billing_account;
pub mod change_set;
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
pub mod node_positions;
pub mod organization;
pub mod output_line;
pub mod resource;
pub mod schematic;
pub mod secret;
pub mod session;
pub mod si_storable;
pub mod support;
pub mod system;
pub mod user;
pub mod workspace;

pub use api_client::{ApiClaim, ApiClient, ApiClientError, ApiClientKind, ApiClientResult};
pub use application::{
    ApplicationContext, ApplicationEntities, ApplicationError, ApplicationListEntry,
    ApplicationResult,
};
pub use billing_account::{BillingAccount, BillingAccountError, BillingAccountResult};
pub use change_set::{ChangeSet, ChangeSetError, ChangeSetResult, ChangeSetStatus, SiChangeSet};
pub use edge::{Edge, EdgeError, EdgeKind, EdgeResult, Vertex};
pub use edit_session::{EditSession, EditSessionError, EditSessionResult};
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
pub use organization::{Organization, OrganizationError};
pub use output_line::{OutputLine, OutputLineStream};
pub use resource::{Resource, ResourceError, ResourceHealth, ResourceResult, ResourceStatus};
pub use schematic::{Schematic, SchematicError, SchematicResult};
pub use secret::{
    EncryptedSecret, Secret, SecretAlgorithm, SecretError, SecretKind, SecretObjectType,
    SecretResult, SecretVersion,
};
pub use session::{SessionError, SessionResult};
pub use si_storable::{MinimalStorable, SiStorable, SimpleStorable};
pub use support::veritech::{Veritech, VeritechError};
pub use system::{SystemError, SystemResult};
pub use user::{LoginReply, LoginRequest, SiClaims, User, UserError, UserResult};
pub use workspace::{Workspace, WorkspaceError};

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("migration error: {0}")]
    Migration(#[from] PgError),
}
pub type ModelResult<T> = Result<T, ModelError>;

pub async fn migrate(pg: &PgPool) -> ModelResult<()> {
    let result = pg.migrate(embedded::migrations::runner()).await?;
    Ok(result)
}

pub fn generate_name(name: Option<String>) -> String {
    if name.is_some() {
        return name.unwrap();
    }
    let mut name_generator = names::Generator::with_naming(names::Name::Numbered);
    let name = name_generator.next().unwrap();
    return name;
}

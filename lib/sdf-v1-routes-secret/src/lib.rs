use axum::{
    Router,
    routing::{
        delete,
        get,
        patch,
        post,
    },
};
use dal::{
    ChangeSetError,
    KeyPairError,
    SecretId,
    StandardModelError,
    TransactionsError,
    UserError,
    WorkspacePk,
    WsEventError,
};
use sdf_core::{
    app_state::AppState,
    impl_default_error_into_response,
};
use telemetry::prelude::*;
use thiserror::Error;

pub mod create_secret;
pub mod delete_secret;
pub mod get_public_key;
pub mod list_secrets;
pub mod update_secret;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SecretError {
    #[error("can't delete a secret with components connected: {0}")]
    CantDeleteSecret(SecretId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("dal secret error: {0}")]
    DalSecret(#[from] dal::SecretError),
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("secret definition view error: {0}")]
    SecretDefinitionView(#[from] dal::SecretDefinitionViewError),
    #[error("secret view error: {0}")]
    SecretView(#[from] dal::SecretViewError),
    #[error("definition not found for secret: {0}")]
    SecretWithInvalidDefinition(SecretId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SecretResult<T> = Result<T, SecretError>;

impl_default_error_into_response!(SecretError);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/get_public_key", get(get_public_key::get_public_key))
        .route("/", post(create_secret::create_secret))
        .route("/", get(list_secrets::list_secrets))
        .route("/", patch(update_secret::update_secret))
        .route("/", delete(delete_secret::delete_secret))
}

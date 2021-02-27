use serde::{Deserialize, Serialize};
use thiserror::Error;
use warp::http::StatusCode;
use warp::{reject::Reject, Rejection, Reply};

use std::convert::Infallible;

use si_data::{NatsTxnError, PgTxn};
use si_model::{
    ApiClientError, ApplicationError, BillingAccountError, ChangeSetError, EdgeError,
    EditSessionError, EntityError, EventError, EventLogError, JwtKeyError, KeyPairError,
    ModelError, NodeError, NodePositionError, OrganizationError, SchematicError, SecretError,
    SessionError, UserError, WorkspaceError,
};

pub mod application_context_dal;
pub mod application_dal;
pub mod attribute_dal;
pub mod cli;
pub mod editor_dal;
pub mod schematic_dal;
pub mod secret_dal;
pub mod session_dal;
pub mod signup_dal;
pub mod updates;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("invalid json pointer: {0}")]
    InvalidJsonPointer(String),
    #[error("invalid json value: {0}")]
    InvalidJsonValue(#[from] serde_json::Error),
    #[error("mismatched json value: {0}")]
    MismatchedJsonValue(String),
    #[error("error in the model layer: {0}")]
    Model(#[from] ModelError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edit session error: {0}")]
    EditSession(#[from] EditSessionError),
    #[error("billing account error: {0}")]
    BillingAccount(#[from] BillingAccountError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("call is unauthorized")]
    Unauthorized,
    #[error("jwt error fetching signing key: {0}")]
    JwtKey(#[from] JwtKeyError),
    #[error("error signing jwt claim: {0}")]
    JwtClaim(String),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("event error: {0}")]
    Event(#[from] EventError),
    #[error("event log error: {0}")]
    EventLog(#[from] EventLogError),
    #[error("api client error: {0}")]
    ApiClient(#[from] ApiClientError),
    #[error("invalid request")]
    InvalidRequest,
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("organization error: {0}")]
    Organization(#[from] OrganizationError),
    #[error("item not found")]
    NotFound,
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("schematic error: {0}")]
    Schematic(#[from] SchematicError),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("pg error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("session error: {0}")]
    Session(#[from] SessionError),
    #[error("application error: {0}")]
    Application(#[from] ApplicationError),
}

pub type HandlerResult<T> = Result<T, HandlerError>;

impl Reject for HandlerError {}

/// An API error serializable to JSON.
#[derive(Deserialize, Serialize)]
pub struct HandlerErrorReply {
    error: HandlerErrorCause,
}

impl HandlerErrorReply {
    fn new(code: impl Into<u16>, message: impl Into<String>) -> Self {
        Self {
            error: HandlerErrorCause {
                code: code.into(),
                message: message.into(),
            },
        }
    }

    pub fn into_cause(self) -> HandlerErrorCause {
        self.error
    }
}

#[derive(Deserialize, Serialize)]
pub struct HandlerErrorCause {
    pub code: u16,
    pub message: String,
}

pub async fn authenticate_api_client(
    txn: &PgTxn<'_>,
    token: impl AsRef<str>,
) -> HandlerResult<si_model::ApiClaim> {
    let claims = si_model::api_client::authenticate(txn, token).await?;
    Ok(claims)
}

pub async fn authenticate(
    txn: &PgTxn<'_>,
    token: impl AsRef<str>,
) -> HandlerResult<si_model::SiClaims> {
    let claims = si_model::user::authenticate(txn, token).await?;
    Ok(claims)
}

pub async fn validate_tenancy(
    txn: &PgTxn<'_>,
    table: impl AsRef<str>,
    id: impl AsRef<str>,
    billing_account_id: impl AsRef<str>,
) -> HandlerResult<()> {
    let table = table.as_ref();
    let id = id.as_ref();
    let billing_account_id = billing_account_id.as_ref();
    let sql = format!("SELECT true AS validate FROM {table} WHERE billing_account_id = si_id_to_primary_key_v1($2) AND id = si_id_to_primary_key_v1($1) LIMIT 1;", table=table);
    let _row = txn
        .query_one(&sql[..], &[&id, &billing_account_id])
        .await
        .map_err(|_| HandlerError::Unauthorized)?;
    Ok(())
}

pub async fn authorize(
    txn: &PgTxn<'_>,
    user_id: impl AsRef<str>,
    subject: impl AsRef<str>,
    action: impl AsRef<str>,
) -> HandlerResult<()> {
    si_model::user::authorize(txn, user_id, subject, action)
        .await
        .map_err(|_| HandlerError::Unauthorized)?;
    Ok(())
}

pub async fn authorize_api_client(
    txn: &PgTxn<'_>,
    api_client_id: impl AsRef<str>,
    subject: impl AsRef<str>,
    action: impl AsRef<str>,
) -> HandlerResult<()> {
    si_model::api_client::authorize(txn, api_client_id, subject, action)
        .await
        .map_err(|_| HandlerError::Unauthorized)?;
    Ok(())
}

// This function receives a `Rejection` and tries to return a custom
// value, otherwise simply passes the rejection along.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code: StatusCode;
    let message: String;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if let Some(HandlerError::NotFound) = err.find() {
        code = StatusCode::NOT_FOUND;
        message = String::from("cannot find item");
    //    } else if let Some(HandlerError::Database(err)) = err.find() {
    //        code = StatusCode::INTERNAL_SERVER_ERROR;
    //       message = err.to_string();
    } else if let Some(HandlerError::Unauthorized) = err.find() {
        code = StatusCode::UNAUTHORIZED;
        message = String::from("request is unauthorized");
    } else if let Some(HandlerError::BillingAccount(BillingAccountError::AccountExists)) =
        err.find()
    {
        code = StatusCode::BAD_REQUEST;
        message = String::from("cannot create billing account");
    } else if let Some(HandlerError::Edge(EdgeError::EdgeExists)) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = String::from("edge already exists");
    } else if let Some(header) = err.find::<warp::reject::MissingHeader>() {
        code = StatusCode::UNAUTHORIZED;
        message = format!("{}", header);
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("unhandled error: {:?}", err);
    }

    dbg!(("returning error code", &message, &code, &err));

    let json = warp::reply::json(&HandlerErrorReply::new(code, message));

    Ok(warp::reply::with_status(json, code))
}

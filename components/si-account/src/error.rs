use si_data;
use thiserror::Error;
use tonic::{self, Response};
use tracing::error;

pub type Result<T> = std::result::Result<T, AccountError>;
pub type TonicResult<T> = std::result::Result<Response<T>, tonic::Status>;

#[derive(Error, Debug)]
pub enum AccountError {
    #[error("this request is not allowed")]
    Authorization,
    #[error("invalid authentication; bad or missing headers")]
    InvalidAuthentication,
    #[error("invalid grpc header; cannot become a string: {0}")]
    GrpcHeaderToString(#[from] tonic::metadata::errors::ToStrError),
    #[error("cannot hash the password")]
    PasswordHash,
    #[error("unknown tenant id")]
    UnknownTenantId(si_data::error::DataError),
    #[error("invalid tenant id for scoped authorization")]
    InvalidTenantId,
    #[error("error with database request: {0})")]
    Db(#[from] si_data::error::DataError),
    #[error("error converting bytes to utf-8 string: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Missing required field {0}")]
    MissingField(String),
    #[error("Couchbase error: {0}")]
    CouchbaseError(#[from] couchbase::CouchbaseError),
    #[error("Invalid JSON type; expected an object, but did not receive one")]
    InvalidJsonObject,
    #[error("Change Set MQTT Agent Error: {0}")]
    ChangeSetAgentClient(#[from] crate::change_set_agent::client::ChangeSetAgentClientError),
    #[error("Change Set Entity Event timeout")]
    ChangeSetEntityEventTimeout,
}

impl From<AccountError> for tonic::Status {
    fn from(err: AccountError) -> tonic::Status {
        match err {
            AccountError::InvalidTenantId | AccountError::MissingField(_) => {
                error!(?err, "tonic invalid argument");
                tonic::Status::new(tonic::Code::InvalidArgument, err.to_string())
            }
            AccountError::Authorization => {
                error!(?err, "tonic permission denied");
                tonic::Status::new(tonic::Code::PermissionDenied, err.to_string())
            }
            AccountError::Db(err) => {
                error!(?err, "tonic failed precondition");
                tonic::Status::new(tonic::Code::FailedPrecondition, err.to_string())
            }
            _ => {
                error!(?err, "tonic unknown");
                tonic::Status::new(tonic::Code::Unknown, err.to_string())
            }
        }
    }
}

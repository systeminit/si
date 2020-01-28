use si_data;
use thiserror::Error;
use tonic::{self, Response};

pub type Result<T> = std::result::Result<T, SshKeyError>;
pub type TonicResult<T> = std::result::Result<Response<T>, tonic::Status>;

#[derive(Error, Debug)]
pub enum SshKeyError {
    #[error("this request is not allowed")]
    Authorization,
    #[error("this request required a user object, but it did not exist")]
    EmptyUser,
    #[error("this request required a billing account object, but it did not exist")]
    EmptyBillingAccount,
    #[error("this request required a component object id, but it did not exist")]
    ComponentMissing,
    #[error("invalid object; missing displayName field")]
    InvalidMissingDisplayName,
    #[error("invalid object; missing name field")]
    InvalidMissingName,
    #[error("invalid object; missing integrationId field")]
    InvalidMissingIntegrationId,
    #[error("cannot find billing account")]
    BillingAccountMissing,
    #[error("cannot find workspace")]
    WorkspaceMissing,
    #[error("invalid authentication; bad or missing headers")]
    InvalidAuthentication,
    #[error("error listing components: {0}")]
    ListComponentsError(si_data::error::DataError),
    #[error("invalid grpc header; cannot become a string: {0}")]
    GrpcHeaderToString(#[from] tonic::metadata::errors::ToStrError),
    #[error("unknown tenant id")]
    UnknownTenantId(si_data::error::DataError),
    #[error("error with database request: {0})")]
    Db(#[from] si_data::error::DataError),
    #[error("error converting bytes to utf-8 string: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("pick component error: {0}")]
    PickComponent(String),
    #[error("invalid key type value")]
    KeyTypeInvalid,
    #[error("invalid key format value")]
    KeyFormatInvalid,
    #[error("invalid bits value for key type: {0} {1}")]
    BitsInvalid(String, u32),
    #[error("unknown tenant id")]
    CreateEntity(si_data::error::DataError),
}

impl From<SshKeyError> for tonic::Status {
    fn from(err: SshKeyError) -> tonic::Status {
        match err {
            SshKeyError::InvalidMissingDisplayName
            | SshKeyError::InvalidMissingName
            | SshKeyError::PickComponent(_)
            | SshKeyError::ComponentMissing => {
                tonic::Status::new(tonic::Code::InvalidArgument, err.to_string())
            }
            SshKeyError::Authorization => {
                tonic::Status::new(tonic::Code::PermissionDenied, err.to_string())
            }
            _ => tonic::Status::new(tonic::Code::Unknown, err.to_string()),
        }
    }
}

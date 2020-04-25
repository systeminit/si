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
    #[error("this request required a user object, but it did not exist")]
    EmptyUser,
    #[error("this request required a billing account object, but it did not exist")]
    EmptyBillingAccount,
    #[error("error creating a user: {0}")]
    CreateUserError(si_data::error::DataError),
    #[error("error creating a integration instance: {0}")]
    CreateIntegrationInstanceError(si_data::error::DataError),
    #[error("error creating a billing account: {0}")]
    CreateBillingAccountError(si_data::error::DataError),
    #[error("error creating a group: {0}")]
    CreateGroupError(si_data::error::DataError),
    #[error("error creating an organization: {0}")]
    CreateOrganizationError(si_data::error::DataError),
    #[error("error creating a workspace: {0}")]
    CreateWorkspaceError(si_data::error::DataError),
    #[error("invalid create user request: {0}")]
    InvalidCreateUserRequest(String),
    #[error("an email was provided, but it wasn't valid - no at sign?")]
    InvalidEmail,
    #[error("invalid object; missing billingAccountId field")]
    InvalidMissingBillingAccountId,
    #[error("invalid object; missing email field")]
    InvalidMissingEmail,
    #[error("invalid object; missing domain field")]
    InvalidMissingDomain,
    #[error("invalid object; missing displayName field")]
    InvalidMissingDisplayName,
    #[error("invalid object; missing name field")]
    InvalidMissingName,
    #[error("invalid object; missing password field")]
    InvalidMissingPassword,
    #[error("invalid object; missing givenName field")]
    InvalidMissingGivenName,
    #[error("invalid object; missing familyName field")]
    InvalidMissingFamilyName,
    #[error("invalid object; missing shortName field")]
    InvalidMissingShortName,
    #[error("invalid object; missing integrationId field")]
    InvalidMissingIntegrationId,
    #[error("invalid object; missing integrationOptionValues field")]
    InvalidMissingIntegrationOptionValues,
    #[error("cannot find billing account")]
    BillingAccountMissing,
    #[error("invalid authentication; bad or missing headers")]
    InvalidAuthentication,
    #[error("invalid grpc header; cannot become a string: {0}")]
    GrpcHeaderToString(#[from] tonic::metadata::errors::ToStrError),
    #[error("login failed")]
    LoginFailed,
    #[error("error listing users: {0}")]
    ListUsersError(si_data::error::DataError),
    #[error("error listing workspaces: {0}")]
    ListWorkspacesError(si_data::error::DataError),
    #[error("error listing organizations: {0}")]
    ListOrganizationsError(si_data::error::DataError),
    #[error("error listing integrations: {0}")]
    ListIntegrationsError(si_data::error::DataError),
    #[error("error listing integration services: {0}")]
    ListIntegrationServicesError(si_data::error::DataError),
    #[error("error listing integration instances: {0}")]
    ListIntegrationInstancesError(si_data::error::DataError),
    #[error("cannot hash the password")]
    PasswordHash,
    #[error("cannot find user")]
    UserMissing,
    #[error("cannot find organization")]
    OrganizationMissing,
    #[error("cannot find workspace")]
    WorkspaceMissing,
    #[error("cannot find integration")]
    IntegrationMissing,
    #[error("cannot find integration service")]
    IntegrationServiceMissing,
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
}

impl From<AccountError> for tonic::Status {
    fn from(err: AccountError) -> tonic::Status {
        match err {
            AccountError::InvalidEmail
            | AccountError::InvalidMissingEmail
            | AccountError::InvalidMissingBillingAccountId
            | AccountError::InvalidMissingDomain
            | AccountError::InvalidMissingDisplayName
            | AccountError::InvalidMissingGivenName
            | AccountError::InvalidMissingFamilyName
            | AccountError::InvalidMissingShortName
            | AccountError::InvalidTenantId
            | AccountError::EmptyBillingAccount
            | AccountError::EmptyUser => {
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

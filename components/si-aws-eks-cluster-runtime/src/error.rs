use paho_mqtt as mqtt;
use si_data;
use thiserror::Error;
use tonic::{self, Response};

use crate::agent::CommandResult;

pub type Result<T> = std::result::Result<T, AwsEksClusterRuntimeError>;
pub type TonicResult<T> = std::result::Result<Response<T>, tonic::Status>;

#[derive(Error, Debug)]
pub enum AwsEksClusterRuntimeError {
    #[error("this request is not allowed")]
    Authorization,
    #[error("this request required a user object, but it did not exist")]
    EmptyUser,
    #[error("this request required a billing account object, but it did not exist")]
    EmptyBillingAccount,
    #[error("this request required a component object id, but it did not exist")]
    ComponentMissing,
    #[error("this request required an entity object id, but it did not exist")]
    EntityMissing,
    #[error("invalid object; missing displayName field")]
    InvalidMissingDisplayName,
    #[error("invalid object; missing name field")]
    InvalidMissingName,
    #[error("invalid object; missing integrationId field")]
    InvalidMissingIntegrationId,
    #[error("invalid object; missing actionName field")]
    InvalidMissingActionName,
    #[error("cannot find billing account")]
    BillingAccountMissing,
    #[error("cannot find workspace")]
    WorkspaceMissing,
    #[error("invalid authentication; bad or missing headers")]
    InvalidAuthentication,
    #[error("error listing components: {0}")]
    ListComponentsError(si_data::error::DataError),
    #[error("error listing entities: {0}")]
    ListEntitiesError(si_data::error::DataError),
    #[error("error listing entity events: {0}")]
    ListEntityEventsError(si_data::error::DataError),
    #[error("invalid grpc header; cannot become a string: {0}")]
    GrpcHeaderToString(#[from] tonic::metadata::errors::ToStrError),
    #[error("unknown tenant id")]
    UnknownTenantId(si_data::error::DataError),
    #[error("error with database request: {0})")]
    Db(#[from] si_data::error::DataError),
    #[error("error converting bytes to utf-8 string: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("error converting bytes to utf-8 string: {0}")]
    Utf8StringError(#[from] std::string::FromUtf8Error),
    #[error("pick component error: {0}")]
    PickComponent(String),
    #[error("invalid key type value")]
    KeyTypeInvalid,
    #[error("invalid key format value")]
    KeyFormatInvalid,
    #[error("invalid bits value for key type: {0} {1}")]
    BitsInvalid(String, u32),
    #[error("error creating an entity: {0}")]
    CreateEntity(si_data::error::DataError),
    #[error("error creating an entity event: {0}")]
    CreateEntityEvent(si_data::error::DataError),
    #[error("mqtt failed: {0}")]
    MqttError(#[from] mqtt::errors::MqttError),
    #[error("protobuf serialization failed: {0}")]
    ProtoError(#[from] prost::EncodeError),
    #[error("invalid entity event; it is missing an input entity")]
    InvalidEntityEventMissingInputEntity,
    #[error("invalid entity event; invalid action name")]
    InvalidEntityEventInvalidActionName,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("no io pipe - this is, um, bad")]
    NoIoPipe,
    #[error("entity event action call is missing an input entity")]
    MissingInputEntity,
    #[error("entity event action call is missing an output entity")]
    MissingOutputEntity,
    #[error("oneshot channel error: {0}")]
    Oneshot(#[from] tokio::sync::oneshot::error::TryRecvError),
    #[error("command failed: {0:?}")]
    CommandFailed(CommandResult),
    #[error("expected output and recevied none")]
    CommandExpectedOutput,
    #[error("grpc client error: {0}")]
    GrpcClient(#[from] tonic::transport::Error),
    #[error("grpc call error: {0}")]
    GrpcCall(#[from] tonic::Status),
    #[error("an external API request has failed")]
    ExternalRequest,
}

impl From<AwsEksClusterRuntimeError> for tonic::Status {
    fn from(err: AwsEksClusterRuntimeError) -> tonic::Status {
        match err {
            AwsEksClusterRuntimeError::InvalidMissingDisplayName
            | AwsEksClusterRuntimeError::InvalidMissingName
            | AwsEksClusterRuntimeError::PickComponent(_)
            | AwsEksClusterRuntimeError::ComponentMissing
            | AwsEksClusterRuntimeError::EntityMissing => {
                tonic::Status::new(tonic::Code::InvalidArgument, err.to_string())
            }
            AwsEksClusterRuntimeError::Authorization => {
                tonic::Status::new(tonic::Code::PermissionDenied, err.to_string())
            }
            _ => tonic::Status::new(tonic::Code::Unknown, err.to_string()),
        }
    }
}

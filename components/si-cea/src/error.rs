use prost;
use si_account::error::AccountError;
use si_data::error::DataError;
use si_settings::error::SettingsError;
use thiserror::Error;
use tonic;
use tracing;

pub type CeaResult<T> = std::result::Result<T, CeaError>;
pub type TonicResult<T> = std::result::Result<tonic::Response<T>, tonic::Status>;

#[derive(Error, Debug)]
pub enum CeaError {
    // CEA
    #[error("action name is invalid: {0}")]
    InvalidActionName(String),
    #[error("authentication invalid; request is missing user id header")]
    InvalidAuthenticationMissingUserId,
    #[error("authentication invalid; request is missing billing account id header")]
    InvalidAuthenticationMissingBillingAccountId,
    #[error("missing input entity in entity event")]
    MissingInputEntity,
    #[error("missing output entity in entity event")]
    MissingOutputEntity,
    #[error("error validating item for insertion: {0}")]
    ValidationError(String),
    #[error("error picking a component: {0}")]
    PickComponent(String),
    #[error("external request has failed")]
    ExternalRequest,
    #[error("command expected output, and has none")]
    CommandExpectedOutput,
    #[error("conversion error: {0}")]
    ConversionError(Box<dyn std::error::Error + Send + Sync>),
    #[error("action error: {0}")]
    ActionError(Box<dyn std::error::Error + Send + Sync>),
    #[error("request is missing a required prop value")]
    RequestMissingProp,

    #[error("tracing error: {0}")]
    TracingError(Box<dyn std::error::Error + Send + Sync>),

    #[error("missing finalize key; this is a programmer error!")]
    MissingFinalizeKey,

    #[error("entity is missing constraints object")]
    MissingEntityConstraints,
    #[error("entity is missing implicit_constraints object")]
    MissingEntityImplicitConstraints,
    #[error("entity is missing properties object")]
    MissingEntityProperties,
    #[error("entity is missing si_properties object")]
    MissingEntitySiProperties,
    #[error("entity is missing si_storable object")]
    MissingEntitySiStorable,

    #[error("create entity request invalid; missing {0}")]
    InvalidEntityCreateRequestMissingField(&'static str),
    #[error("edit entity property request invalid; missing entity_id")]
    InvalidEntityEditRequestMissingId,
    #[error("edit entity property request invalid; missing property")]
    InvalidEntityEditRequestMissingProperty,
    #[error("get entity request invalid; missing entity_id")]
    InvalidEntityGetRequestMissingId,
    #[error("get component request invalid; missing component_id")]
    InvalidComponentGetRequestMissingId,
    #[error("pick component request invalid; missing constraints")]
    InvalidComponentPickRequestMissingConstraints,

    // Prost
    #[error("prost failed to encode: {0}")]
    ProstEncodeError(#[from] prost::EncodeError),

    #[error("{0}")]
    Infallible(#[from] std::convert::Infallible),

    // Tonic
    #[error("tonic failed to convert metadata to string: {0}")]
    TonicToString(#[from] tonic::metadata::errors::ToStrError),
    #[error("tonic client call failed: {0}")]
    TonicStatus(#[from] tonic::Status),

    // SI Data
    #[error("data operation failed: {0}")]
    DataError(#[from] DataError),

    // SI Account
    #[error("account operation failed: {0}")]
    AccountError(#[from] AccountError),

    // SI Settings
    #[error("settings operation failed: {0}")]
    SettingsError(#[from] SettingsError),

    // SI Transport
    #[error("transport operation failed: {0}")]
    TransportError(#[from] si_transport::Error),

    // Tonic
    #[error("transport error: {0}")]
    TonicTransportError(#[from] tonic::transport::Error),

    // Tracing
    #[error("cannot configure global default tracing subscriber: {0}")]
    TracingGlobalError(#[from] tracing::subscriber::SetGlobalDefaultError),

    // IO
    #[error("error with an I/O operation: {0}")]
    IO(#[from] std::io::Error),

    // String Conversion
    #[error("cannot convert string to utf-8: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    // Serde
    #[error("serde cannot convert json from a string: {0}")]
    JsonString(#[from] serde_json::error::Error),
    #[error("serde cannot serialize/deserialze to yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

impl CeaError {
    pub fn conversion_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::ConversionError(Box::new(error))
    }

    pub fn action_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::ActionError(Box::new(error))
    }
}

impl From<CeaError> for tonic::Status {
    fn from(err: CeaError) -> tonic::Status {
        match err {
            CeaError::AccountError(_) => {
                tonic::Status::new(tonic::Code::PermissionDenied, err.to_string())
            }
            CeaError::PickComponent(_) => {
                tonic::Status::new(tonic::Code::InvalidArgument, err.to_string())
            }
            _ => tonic::Status::new(tonic::Code::Unknown, err.to_string()),
        }
    }
}

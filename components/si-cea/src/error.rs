use paho_mqtt as mqtt;
use prost;
use thiserror::Error;
use tonic;
use tracing;

use si_account::error::AccountError;
use si_data::error::DataError;
use si_settings::error::SettingsError;

use crate::agent::utility::spawn_command::CommandResult;

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
    #[error("pick component invalid; missing constraints")]
    InvalidPickMissingConstraints,
    #[error("error validating item for insertion: {0}")]
    ValidationError(String),
    #[error("error picking a component: {0}")]
    PickComponent(String),
    #[error("no dispatch function for action - integration service id: {0}, action name: {1}")]
    DispatchFunctionMissing(String, String),
    #[error("external request has failed")]
    ExternalRequest,
    #[error("command failed: {0}")]
    CommandFailed(CommandResult),
    #[error("command expected output, and has none")]
    CommandExpectedOutput,
    #[error("no I/O pipe during command call")]
    NoIoPipe,
    #[error("conversion error: {0}")]
    ConversionError(Box<dyn std::error::Error + Send + Sync>),
    #[error("action error: {0}")]
    ActionError(String),
    #[error("request is missing a required prop value")]
    RequestMissingProp,

    // MQTT
    #[error("mqtt failed: {0}")]
    Mqtt(#[from] mqtt::errors::MqttError),

    // Prost
    #[error("prost failed to encode: {0}")]
    ProstEncodeError(#[from] prost::EncodeError),

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

    // Tokio
    #[error("channel send error: {0}")]
    CommandOuputChannelSend(
        #[from]
        tokio::sync::mpsc::error::SendError<crate::agent::utility::spawn_command::OutputLine>,
    ),

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

    pub fn action_error(msg: impl Into<String>) -> Self {
        Self::ActionError(msg.into())
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

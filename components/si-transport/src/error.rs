use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("receiver has closed or dropped the receiving end of the channel")]
    ChannelClosed,
    #[error("{0}")]
    Infallible(#[from] std::convert::Infallible),
    #[error("invalid agent command kind: {0}")]
    InvalidAgentCommand(String),
    #[error("invalid agent data kind: {0}")]
    InvalidAgentData(String),
    #[error("invalid header or topic format: {0}")]
    InvalidHeaderOrTopic(String),
    #[error("invalid header format, should not have shared prefix: {0}")]
    InvalidHeaderShared(String),
    #[error("invalid qos kind: {0}")]
    InvalidQoS(i32),
    #[error("json serialize/deserialize error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing content type property in mqtt message")]
    MissingContentType,
    #[error("missing or empty header part {0} for string: {1}")]
    MissingHeaderOrTopicPart(&'static str, String),
    #[error("mqtt error: {0}")]
    Mqtt(#[from] paho_mqtt::Error),
}

impl Error {
    pub(crate) fn missing_part(part: &'static str, s: impl Into<String>) -> Self {
        Self::MissingHeaderOrTopicPart(part, s.into())
    }
}

use thiserror::Error;

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// An error when parsing content info metadata.
#[derive(Debug, Error)]
#[error("content info error: {0}")]
pub struct ContentInfoError(#[source] pub BoxError);

impl ContentInfoError {
    /// Returns an error with the provided error as its source.
    pub fn from_err<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self(Box::new(err))
    }
}

/// An error when parsing content info metadata from NATS message headers.
#[cfg(feature = "nats-headers")]
#[derive(Debug, Error)]
pub enum HeaderMapParseMessageInfoError {
    /// When a required header is missing
    #[error("missing nats header: {0}")]
    MissingHeader(&'static str),
    /// When no headers are found
    #[error("nats headers are required but none were found")]
    MissingHeaders,
    /// When a message version can't be parsed as an integer
    #[error("error parsing message version header: {0}")]
    ParseVersion(#[source] std::num::ParseIntError),
}

/// An error when serializing a message.
#[cfg(feature = "serialize")]
#[derive(Debug, Error)]
#[error("error serializing message: {0}")]
pub struct SerializeError(#[source] BoxError);

#[cfg(feature = "serialize")]
impl SerializeError {
    /// Returns an error with the provided error as its source.
    pub fn from_err<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self(Box::new(err))
    }
}

/// An error when deserializing into a message.
#[cfg(feature = "deserialize")]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum DeserializeError {
    /// When deserializing from a slice of bytes encoding in a particular format
    #[error("deserialize error: {0}")]
    Deserialize(#[source] BoxError),
    /// When a message isn't supported based on its reported content type
    #[error("unsupported content type: {0}")]
    UnsupportedContentType(String),
    /// When a message fails to upgrade into a current message version
    #[error("upgrade error: {0}")]
    Upgrade(#[source] BoxError),
}

#[cfg(feature = "deserialize")]
impl DeserializeError {
    /// Returns a new deserialize variant of the error.
    pub fn deserialize<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Deserialize(Box::new(err))
    }

    /// Returns a new upgrade variant of the error.
    pub fn upgrade<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Upgrade(Box::new(err))
    }
}

/// An error when upgrading a message into its current version.
#[derive(Debug, Error)]
#[error("error upgrading message: {0}")]
pub struct UpgradeError(#[source] BoxError);

impl UpgradeError {
    /// Returns an error with the provided error as its source.
    pub fn from_err<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self(Box::new(err))
    }
}

/// An error when serializing a message to a content type that isn't supported.
#[derive(Debug, Error)]
#[error("unsupported default content type: {0}")]
pub struct UnsupportedDefaultContentTypeError(pub(crate) String);

/// An error when determining the strategy to deserialize into a message.
#[cfg(feature = "deserialize")]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum NegotiateError {
    /// When failing to parse content info metadata
    #[error("error parsing content info: {0}")]
    ContentInfo(#[from] ContentInfoError),
    /// When failing to deserialize the message from bytes
    #[error("error deserializing message: {0}")]
    Deserialize(#[from] DeserializeError),
    /// When a provided content type string is not supported for a message type
    #[error("unsupported content type: {0}")]
    UnsupportedContentType(String),
    /// When a provided message type string is not supported for a message type
    #[error("unsupported message type: {0}")]
    UnsupportedMessageType(String),
    /// When a provided message version is not supported for a message type
    #[error("unsupported message version: {0}")]
    UnsupportedMessageVersion(u64),
}

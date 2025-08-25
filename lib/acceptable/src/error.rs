use thiserror::Error;

/// Alias for a type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Error)]
#[error("content info error: {0}")]
pub struct ContentInfoError(#[source] pub BoxError);

impl ContentInfoError {
    #[allow(missing_docs)]
    pub fn from_err<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self(Box::new(err))
    }
}

#[cfg(feature = "nats-headers")]
#[derive(Debug, Error)]
pub enum HeaderMapParseMessageInfoError {
    #[error("missing nats header: {0}")]
    MissingHeader(&'static str),
    #[error("error parsing message version header: {0}")]
    ParseVersion(#[source] std::num::ParseIntError),
}

#[cfg(feature = "serialize")]
#[derive(Debug, Error)]
#[error("error serializing message: {0}")]
pub struct SerializeError(#[source] BoxError);

#[cfg(feature = "serialize")]
impl SerializeError {
    #[allow(missing_docs)]
    pub fn from_err<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self(Box::new(err))
    }
}

#[cfg(feature = "deserialize")]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum DeserializeError {
    #[error("deserialize error: {0}")]
    Deserialize(#[source] BoxError),
    #[error("unsupported content type: {0}")]
    UnsupportedContentType(String),
    #[error("upgrade error: {0}")]
    Upgrade(#[source] BoxError),
}

#[cfg(feature = "deserialize")]
impl DeserializeError {
    pub fn deserialize<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Deserialize(Box::new(err))
    }

    pub fn upgrade<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Upgrade(Box::new(err))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
#[error("error upgrading message: {0}")]
pub struct UpgradeError(#[source] BoxError);

impl UpgradeError {
    #[allow(missing_docs)]
    pub fn from_err<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self(Box::new(err))
    }
}

#[allow(missing_docs)]
#[derive(Debug, Error)]
#[error("unsupported content type: {0}")]
pub struct UnsupportedContentTypeError(pub(crate) String);

#[allow(missing_docs)]
#[derive(Debug, Error)]
#[error("unsupported default content type: {0}")]
pub struct UnsupportedDefaultContentTypeError(pub(crate) String);

#[cfg(feature = "deserialize")]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum NegotiateError {
    #[error("error parsing content info: {0}")]
    ContentInfo(#[from] ContentInfoError),
    #[error("error deserializing message: {0}")]
    Deserialize(#[from] DeserializeError),
    #[error("unsupported content type: {0}")]
    UnsupportedContentType(String),
    #[error("unsupported message type: {0}")]
    UnsupportedMessageType(String),
    #[error("unsupported message version: {0}")]
    UnsupportedMessageVersion(u64),
}

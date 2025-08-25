#[cfg(feature = "deserialize")]
use crate::error::DeserializeError;
#[cfg(feature = "serialize")]
use crate::error::{
    SerializeError,
    UnsupportedDefaultContentTypeError,
};
use crate::{
    CONTENT_TYPE_CBOR,
    CONTENT_TYPE_JSON,
    all_versions::AllVersions,
    error::UpgradeError,
    id::RequestId,
    versioned::Versioned,
};

/// A container enclosing a current version of the message type.
pub trait Container {
    /// The [`Versioned`] type, representing the current version of the message.
    type Current: Versioned;

    /// The associated "all versions" type which includes the message version upgrade behavior.
    type AllVersions: AllVersions + IntoContainer<Container = Self>;

    /// The message type string, used when serializing and deserializing the message.
    const MESSAGE_TYPE: &'static str;

    /// Builds a new instance of `Self`, enclosing a current version of this message type.
    fn new(current: Self::Current) -> Self;

    /// The unique ID of this message.
    fn id(&self) -> RequestId;

    /// The message type string for the message type.
    fn message_type() -> &'static str;

    /// The default content type string for the message type.
    #[inline]
    fn default_content_type() -> &'static str {
        CONTENT_TYPE_CBOR
    }

    /// The current message version for the message type.
    #[inline]
    fn message_version() -> u64 {
        <Self::Current as Versioned>::message_version()
    }

    /// The current message version for this message.
    #[inline]
    fn version(&self) -> u64 {
        Self::message_version()
    }
}

/// Trait to help determine if a message payload is supported for deserializing.
///
/// The trait member functions are typically used when implementing the `Negotiate` trait.
pub trait SupportsContainers {
    /// Whether or not the content type is supported for the message type.
    fn is_content_type_supported(ty: &str) -> bool;

    /// Whether or not the message type is supported for the message type.
    fn is_message_type_supported(ty: &str) -> bool;

    /// Whether or not the message version is supported for the message type.
    fn is_message_version_supported(version: u64) -> bool;
}

impl<T> SupportsContainers for T
where
    T: Container,
{
    #[inline]
    fn is_content_type_supported(ty: &str) -> bool {
        matches!(ty, CONTENT_TYPE_CBOR | CONTENT_TYPE_JSON)
    }

    #[inline]
    fn is_message_type_supported(ty: &str) -> bool {
        Self::message_type() == ty
    }

    #[inline]
    fn is_message_version_supported(version: u64) -> bool {
        T::AllVersions::all_versions().contains(&version)
    }
}

/// Upgrades a message type into the current version, enclosed in a [`Container`]-implementing
/// type.
pub trait IntoContainer {
    /// Associated container type which encloses the current message version.
    type Container: Container<AllVersions = Self>;

    /// Upgrades a message into its current version and returned in a [`Container`]-implementing
    /// type.
    fn into_container(self) -> Result<Self::Container, UpgradeError>;
}

/// Deserialize a message from a slice of bytes.
#[cfg(feature = "deserialize")]
pub trait DeserializeContainer {
    /// Deserialize a message from a slice of bytes containing a JSON message.
    fn from_json_slice(bytes: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    /// Deserialize a message from a slice of bytes containing a CBOR message.
    fn from_cbor_slice(bytes: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    /// Deserialize a message from a slice of bytes, using the `content_type` string to determine
    /// the message serialization.
    fn from_slice(content_type: &str, bytes: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized;
}

#[cfg(feature = "deserialize")]
impl<T> DeserializeContainer for T
where
    T: Container,
    <T as Container>::AllVersions: serde::de::DeserializeOwned,
{
    fn from_json_slice(bytes: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let all_versions: <T as Container>::AllVersions =
            serde_json::from_slice(bytes).map_err(DeserializeError::deserialize)?;

        all_versions
            .into_container()
            .map_err(DeserializeError::upgrade)
    }

    fn from_cbor_slice(bytes: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        let all_versions: <T as Container>::AllVersions =
            ciborium::from_reader(bytes).map_err(DeserializeError::deserialize)?;

        all_versions
            .into_container()
            .map_err(DeserializeError::upgrade)
    }

    fn from_slice(content_type: &str, bytes: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized,
    {
        match content_type {
            CONTENT_TYPE_CBOR => Self::from_cbor_slice(bytes),
            CONTENT_TYPE_JSON => Self::from_json_slice(bytes),
            unexpected => Err(DeserializeError::UnsupportedContentType(
                unexpected.to_string(),
            )),
        }
    }
}

/// Serialize a message into bytes.
#[cfg(feature = "serialize")]
pub trait SerializeContainer {
    /// Serialize a message into bytes containing a JSON message.
    fn to_json_vec(&self) -> Result<Vec<u8>, SerializeError>;

    /// Serialize a message into bytes containing a CBOR message.
    fn to_cbor_vec(&self) -> Result<Vec<u8>, SerializeError>;

    /// Serialize a message into bytes, using internal logic to determine the message
    /// serialization.
    ///
    /// The content type string and message bytes are returned.
    fn to_vec(&self) -> Result<(&'static str, Vec<u8>), SerializeError>;
}

#[cfg(feature = "serialize")]
impl<T> SerializeContainer for T
where
    T: Container + serde::Serialize,
{
    fn to_json_vec(&self) -> Result<Vec<u8>, SerializeError> {
        serde_json::to_vec(self).map_err(SerializeError::from_err)
    }

    fn to_cbor_vec(&self) -> Result<Vec<u8>, SerializeError> {
        let mut bytes = Vec::new();
        ciborium::into_writer(self, &mut bytes).map_err(SerializeError::from_err)?;
        Ok(bytes)
    }

    fn to_vec(&self) -> Result<(&'static str, Vec<u8>), SerializeError> {
        match T::default_content_type() {
            CONTENT_TYPE_CBOR => Ok((CONTENT_TYPE_CBOR, self.to_cbor_vec()?)),
            CONTENT_TYPE_JSON => Ok((CONTENT_TYPE_JSON, self.to_json_vec()?)),
            unexpected => Err(SerializeError::from_err(
                UnsupportedDefaultContentTypeError(unexpected.to_string()),
            )),
        }
    }
}

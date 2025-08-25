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

pub trait Container {
    type Current: Versioned;

    type AllVersions: AllVersions + IntoContainer<Container = Self>;

    const MESSAGE_TYPE: &'static str;

    fn new(current: Self::Current) -> Self;

    fn id(&self) -> RequestId;

    fn message_type() -> &'static str;

    #[inline]
    fn default_content_type() -> &'static str {
        CONTENT_TYPE_CBOR
    }

    #[inline]
    fn message_version() -> u64 {
        <Self::Current as Versioned>::message_version()
    }

    #[inline]
    fn version(&self) -> u64 {
        Self::message_version()
    }
}

pub trait SupportsContainers {
    fn is_content_type_supported(ty: &str) -> bool;

    fn is_message_type_supported(ty: &str) -> bool;

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

pub trait IntoContainer {
    type Container: Container<AllVersions = Self>;

    fn into_container(self) -> Result<Self::Container, UpgradeError>;
}

#[cfg(feature = "deserialize")]
pub trait DeserializeContainer {
    fn from_json_slice(bytes: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized;

    fn from_cbor_slice(bytes: &[u8]) -> Result<Self, DeserializeError>
    where
        Self: Sized;

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

#[cfg(feature = "serialize")]
pub trait SerializeContainer {
    fn to_json_vec(&self) -> Result<Vec<u8>, SerializeError>;

    fn to_cbor_vec(&self) -> Result<Vec<u8>, SerializeError>;

    fn to_vec(&self) -> Result<Vec<u8>, SerializeError>;
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

    fn to_vec(&self) -> Result<Vec<u8>, SerializeError> {
        match T::default_content_type() {
            CONTENT_TYPE_CBOR => self.to_cbor_vec(),
            CONTENT_TYPE_JSON => self.to_json_vec(),
            unexpected => Err(SerializeError::from_err(
                UnsupportedDefaultContentTypeError(unexpected.to_string()),
            )),
        }
    }
}

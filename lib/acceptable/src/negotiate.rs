use crate::{
    container::{
        DeserializeContainer,
        SupportsContainers,
    },
    content_info::ContentInfo,
    error::NegotiateError,
};

pub trait Negotiate {
    fn negotiate(content_info: &ContentInfo<'_>, bytes: &[u8]) -> Result<Self, NegotiateError>
    where
        Self: Sized;
}

impl<T> Negotiate for T
where
    T: SupportsContainers + DeserializeContainer,
{
    fn negotiate(content_info: &ContentInfo<'_>, bytes: &[u8]) -> Result<Self, NegotiateError>
    where
        Self: Sized,
    {
        if !Self::is_content_type_supported(content_info.content_type.as_str()) {
            return Err(NegotiateError::UnsupportedContentType(
                content_info.content_type.to_string(),
            ));
        }
        if !Self::is_message_type_supported(content_info.message_type.as_str()) {
            return Err(NegotiateError::UnsupportedMessageType(
                content_info.message_type.to_string(),
            ));
        }
        if !Self::is_message_version_supported(content_info.message_version.as_u64()) {
            return Err(NegotiateError::UnsupportedMessageVersion(
                content_info.message_version.as_u64(),
            ));
        }

        Self::from_slice(content_info.content_type.as_str(), bytes).map_err(Into::into)
    }
}

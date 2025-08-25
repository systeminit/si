use edda_core::api_types::{
    Container,
    ContentInfo,
    Negotiate,
    NegotiateError,
    new_change_set_request::NewChangeSetRequest,
    rebuild_request::RebuildRequest,
    update_request::UpdateRequest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChangeSetRequestKind {
    NewChangeSet(NewChangeSetRequest),
    Update(UpdateRequest),
    Rebuild(RebuildRequest),
}

impl Negotiate for ChangeSetRequestKind {
    fn negotiate(content_info: &ContentInfo<'_>, bytes: &[u8]) -> Result<Self, NegotiateError>
    where
        Self: Sized,
    {
        match content_info.message_type.as_str() {
            UpdateRequest::MESSAGE_TYPE => Ok(ChangeSetRequestKind::Update(
                UpdateRequest::negotiate(content_info, bytes)?,
            )),
            RebuildRequest::MESSAGE_TYPE => Ok(ChangeSetRequestKind::Rebuild(
                RebuildRequest::negotiate(content_info, bytes)?,
            )),
            NewChangeSetRequest::MESSAGE_TYPE => Ok(ChangeSetRequestKind::NewChangeSet(
                NewChangeSetRequest::negotiate(content_info, bytes)?,
            )),
            unsupported => Err(NegotiateError::UnsupportedContentType(
                unsupported.to_string(),
            )),
        }
    }
}

pub type DeploymentRequest = RebuildRequest;

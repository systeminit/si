use std::sync::Arc;

use async_nats::Subject;
use async_trait::async_trait;

use crate::Head;

use super::{
    rejection::{MatchedSubjectMissing, MatchedSubjectRejection},
    FromMessageHead,
};

#[derive(Clone, Debug)]
pub struct MatchedSubject(pub(crate) Arc<str>);

impl MatchedSubject {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<Subject> for MatchedSubject {
    fn from(value: Subject) -> Self {
        Self(value.to_string().into())
    }
}

impl From<&Subject> for MatchedSubject {
    fn from(value: &Subject) -> Self {
        Self(value.to_string().into())
    }
}

impl From<String> for MatchedSubject {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<&str> for MatchedSubject {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

#[async_trait]
impl<S> FromMessageHead<S> for MatchedSubject {
    type Rejection = MatchedSubjectRejection;

    async fn from_message_head(head: &mut Head, _state: &S) -> Result<Self, Self::Rejection> {
        let matched_subject = head
            .extensions
            .get::<Self>()
            .ok_or(MatchedSubjectRejection::MatchedSubjectMissing(
                MatchedSubjectMissing,
            ))?
            .clone();

        Ok(matched_subject)
    }
}

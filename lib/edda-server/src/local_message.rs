use bytes::Bytes;
use naxum::{
    Extensions,
    Head,
    MessageHead,
};
use si_data_nats::{
    HeaderMap,
    Subject,
};

/// Local Naxum-compatible message.
#[derive(Clone, Debug)]
pub struct LocalMessage {
    pub subject: Subject,
    pub headers: Option<HeaderMap>,
    pub payload: Bytes,
}

impl MessageHead for LocalMessage {
    fn subject(&self) -> &Subject {
        &self.subject
    }

    fn reply(&self) -> Option<&Subject> {
        None
    }

    fn headers(&self) -> Option<&HeaderMap> {
        self.headers.as_ref()
    }

    fn status(&self) -> Option<naxum::StatusCode> {
        None
    }

    fn description(&self) -> Option<&str> {
        None
    }

    fn length(&self) -> usize {
        self.payload_length()
    }

    fn payload_length(&self) -> usize {
        self.payload.len()
    }

    fn from_head_and_payload(
        head: naxum::Head,
        payload: Bytes,
    ) -> Result<(Self, naxum::Extensions), naxum::FromPartsError>
    where
        Self: Sized,
    {
        let Head {
            subject,
            reply: _,
            headers,
            status: _,
            description: _,
            length: _,
            extensions,
        } = head;

        Ok((
            Self {
                subject,
                headers,
                payload,
            },
            extensions,
        ))
    }

    fn into_head_and_payload(self) -> (naxum::Head, Bytes) {
        let Self {
            subject,
            headers,
            payload,
        } = self;

        (
            Head {
                subject,
                reply: None,
                headers,
                status: None,
                description: None,
                length: 0,
                extensions: Extensions::new(),
            },
            payload,
        )
    }
}

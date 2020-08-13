use crate::{error::Error, metadata::Header, qos::QoS};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
};

const CONTEXT_TYPE_PREFIX: &str = "application/vnd.si.";
const CONTEXT_TYPE_SUFFIX: &str = "+json";

pub trait TypeHint {
    fn type_name() -> &'static str;
}

#[derive(Debug, Clone)]
pub struct WireMessage {
    pub(crate) header: Header,
    pub(crate) qos: QoS,
    pub(crate) response_header: Option<Header>,
    pub(crate) payload_type: String,
    pub(crate) payload: Vec<u8>,
}

impl WireMessage {
    pub fn from_parts<T>(
        header: Header,
        qos: QoS,
        response_header: Option<Header>,
        payload_type: impl Into<String>,
        payload: T,
    ) -> Result<Self, Error>
    where
        T: Serialize,
    {
        let payload_type = payload_type.into();
        let payload = serde_json::to_vec(&payload)?;

        Ok(Self {
            header,
            qos,
            response_header,
            payload_type,
            payload,
        })
    }

    pub fn payload_type(&self) -> &str {
        &self.payload_type
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn response_header(&self) -> Option<&Header> {
        self.response_header.as_ref()
    }
}

impl TryFrom<paho_mqtt::Message> for WireMessage {
    type Error = Error;

    fn try_from(value: paho_mqtt::Message) -> Result<Self, Self::Error> {
        let header = value.topic().parse()?;
        let qos = value.qos().try_into()?;
        let response_header = match value
            .properties()
            .get_string(paho_mqtt::PropertyCode::ResponseTopic)
        {
            Some(response_header_str) => Some(response_header_str.parse()?),
            None => None,
        };
        let payload_type = payload_type_from_content_type(
            &value
                .properties()
                .get_string(paho_mqtt::PropertyCode::ContentType)
                .ok_or(Error::MissingContentType)?,
        )
        .to_string();
        let payload = value.payload().to_vec();

        Ok(Self {
            header,
            qos,
            response_header,
            payload_type,
            payload,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Message<T> {
    header: Header,
    qos: QoS,
    response_header: Option<Header>,
    payload: T,
}

impl<T> Message<T> {
    pub fn new(
        header: impl Into<Header>,
        qos: QoS,
        response_header: Option<impl Into<Header>>,
        payload: T,
    ) -> Self {
        Message {
            header: header.into(),
            qos,
            response_header: response_header.map(|rt| rt.into()),
            payload,
        }
    }

    pub fn into_parts(self) -> (Header, QoS, Option<Header>, T) {
        (self.header, self.qos, self.response_header, self.payload)
    }
}

impl<T> From<&Message<T>> for Message<T> {
    fn from(value: &Message<T>) -> Self {
        value.into()
    }
}

impl<T> TryFrom<WireMessage> for Message<T>
where
    T: DeserializeOwned,
{
    type Error = Error;

    fn try_from(value: WireMessage) -> Result<Self, Self::Error> {
        let header = value.header;
        let qos = value.qos;
        let response_header = value.response_header;
        let payload: T = serde_json::from_slice(&value.payload)?;

        Ok(Self {
            header,
            qos,
            response_header,
            payload,
        })
    }
}

impl<'a, T> TryFrom<Message<&&'a mut T>> for WireMessage
where
    T: Serialize + TypeHint,
{
    type Error = Error;

    fn try_from(value: Message<&&'a mut T>) -> Result<Self, Self::Error> {
        let header = value.header;
        let qos = value.qos;
        let response_header = value.response_header;
        let payload = serde_json::to_vec(&value.payload)?;
        let payload_type = T::type_name().to_owned();

        Ok(Self {
            header,
            qos,
            response_header,
            payload_type,
            payload,
        })
    }
}

pub(crate) fn content_type(payload_type: impl fmt::Display) -> String {
    format!(
        "{}{}{}",
        CONTEXT_TYPE_PREFIX, payload_type, CONTEXT_TYPE_SUFFIX,
    )
}

fn payload_type_from_content_type(content_type: &str) -> &str {
    content_type
        .trim_start_matches("application/vnd.si.")
        .trim_end_matches("+json")
}

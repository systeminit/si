use crate::{error::Error, metadata::Header, qos::QoS};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    str::FromStr,
};

const CONTEXT_TYPE_PREFIX: &str = "application/vnd.si.";
const CONTEXT_TYPE_SUFFIX: &str = "+json";

pub trait TypeHint {
    fn type_name() -> &'static str;
}

#[derive(Debug, Clone)]
pub struct WireMessage {
    pub(crate) topic: String,
    pub(crate) qos: QoS,
    pub(crate) response_topic: Option<String>,
    pub(crate) payload_type: String,
    pub(crate) payload: Vec<u8>,
}

impl WireMessage {
    pub fn from_parts<T>(
        topic: impl Into<String>,
        qos: QoS,
        response_topic: Option<impl Into<String>>,
        payload_type: impl Into<String>,
        payload: T,
    ) -> Result<Self, Error>
    where
        T: Serialize,
    {
        let topic = topic.into();
        let response_topic = response_topic.map(|rt| rt.into());
        let payload_type = payload_type.into();
        let payload = serde_json::to_vec(&payload)?;

        Ok(Self {
            topic,
            qos,
            response_topic,
            payload_type,
            payload,
        })
    }

    pub fn payload_type(&self) -> &str {
        &self.payload_type
    }

    pub fn topic_str(&self) -> &str {
        &self.topic
    }

    pub fn response_topic_str(&self) -> Option<&str> {
        self.response_topic.as_ref().map(String::as_str)
    }
}

impl TryFrom<paho_mqtt::Message> for WireMessage {
    type Error = Error;

    fn try_from(value: paho_mqtt::Message) -> Result<Self, Self::Error> {
        let topic = value.topic().into();
        let qos = value.qos().try_into()?;
        let response_topic = value
            .properties()
            .get_string(paho_mqtt::PropertyCode::ResponseTopic);
        let payload_type = payload_type_from_content_type(
            &value
                .properties()
                .get_string(paho_mqtt::PropertyCode::ContentType)
                .ok_or(Error::MissingContentType)?,
        )
        .to_string();
        let payload = value.payload().to_vec();

        Ok(Self {
            topic,
            qos,
            response_topic,
            payload_type,
            payload,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Message<T> {
    header: Header,
    qos: QoS,
    response_topic: Option<Header>,
    payload: T,
}

impl<T> Message<T> {
    pub fn new(
        header: impl Into<Header>,
        qos: QoS,
        response_topic: Option<impl Into<Header>>,
        payload: T,
    ) -> Self {
        Message {
            header: header.into(),
            qos,
            response_topic: response_topic.map(|rt| rt.into()),
            payload,
        }
    }

    pub fn into_parts(self) -> (Header, QoS, Option<Header>, T) {
        (self.header, self.qos, self.response_topic, self.payload)
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
        let header = Header::from_str(&value.topic)?;
        let qos = value.qos;
        let response_topic = match value.response_topic {
            Some(ref response_topic_str) => Some(Header::from_str(response_topic_str)?),
            None => None,
        };
        let payload: T = serde_json::from_slice(&value.payload)?;

        Ok(Self {
            header,
            qos,
            response_topic,
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
        let topic = value.header.to_string();
        let qos = value.qos;
        let response_topic = value.response_topic.map(|rt| rt.to_string());
        let payload = serde_json::to_vec(&value.payload)?;
        let payload_type = T::type_name().to_owned();

        Ok(Self {
            topic,
            qos,
            response_topic,
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

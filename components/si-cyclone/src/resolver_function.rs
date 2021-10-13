use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionRequest {
    pub kind: String,
    pub code: String,
    pub container_image: String,
    pub container_tag: String,
}

impl ResolverFunctionRequest {
    pub fn deserialize_from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn serialize_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ResolverFunctionMessage {
    Start,
    Finish,
    Heartbeat,
    Fail(Fail),
    OutputStream(OutputStream),
    FunctionResult(FunctionResult),
}

impl ResolverFunctionMessage {
    pub fn fail(message: impl Into<String>) -> Self {
        Self::Fail(Fail {
            message: message.into(),
        })
    }

    pub fn deserialize_from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn serialize_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ResolverFunctionExecutingMessage {
    Heartbeat,
    OutputStream(OutputStream),
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputStream {
    pub(crate) stream: String,
    pub(crate) level: String,
    pub(crate) group: Option<String>,
    pub(crate) data: Option<Value>,
    pub(crate) message: String,
    pub(crate) timestamp: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum FunctionResult {
    Success(ResultSuccess),
    Failure(ResultFailure),
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResultSuccess {
    pub data: Value,
    pub unset: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResultFailure {
    pub error: ResultFailureError,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResultFailureError {
    pub message: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Fail {
    pub message: String,
}

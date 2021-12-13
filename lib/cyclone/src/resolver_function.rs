use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverFunctionRequest {
    pub execution_id: String,
    pub handler: String,
    pub parameters: Option<HashMap<String, Value>>,
    pub code_base64: String,
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
    Result(ResolverFunctionResult),
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

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ResolverFunctionExecutingMessage {
    Heartbeat,
    OutputStream(OutputStream),
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OutputStream {
    pub stream: String,
    pub execution_id: String,
    pub level: String,
    pub group: Option<String>,
    pub data: Option<Value>,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ResolverFunctionResult {
    Success(ResolverFunctionResultSuccess),
    Failure(ResolverFunctionResultFailure),
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolverFunctionResultSuccess {
    pub execution_id: String,
    pub data: Value,
    pub unset: bool,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolverFunctionResultFailure {
    pub execution_id: String,
    pub error: ResolverFunctionResultFailureError,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolverFunctionResultFailureError {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Fail {
    pub message: String,
}

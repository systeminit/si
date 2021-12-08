use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: Component,
    pub code_base64: String,
}

impl QualificationCheckRequest {
    pub fn deserialize_from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn serialize_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub name: String,
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum QualificationCheckMessage {
    Start,
    Finish,
    Heartbeat,
    Fail(Fail),
    OutputStream(OutputStream),
    Result(QualificationCheckResult),
}

impl QualificationCheckMessage {
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
pub enum QualificationCheckExecutingMessage {
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
pub enum QualificationCheckResult {
    Success(QualificationCheckResultSuccess),
    Failure(QualificationCheckResultFailure),
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QualificationCheckResultSuccess {
    pub execution_id: String,
    pub qualified: bool,
    pub output: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QualificationCheckResultFailure {
    pub execution_id: String,
    pub error: QualificationCheckResultFailureError,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QualificationCheckResultFailureError {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Fail {
    pub message: String,
}

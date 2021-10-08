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

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ResolverFunctionMessage {
    Start,
    Finish,
    Heartbeat,
    OutputStream(OutputStream),
    FunctionResult(FunctionResult),
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
    pub(crate) data: Value,
    pub(crate) unset: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResultFailure {
    pub(crate) error: ResultFailureError,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResultFailureError {
    pub(crate) message: String,
    pub(crate) name: String,
}

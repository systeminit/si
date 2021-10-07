use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResolverFunctionRequest {}

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
pub struct OutputStream;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FunctionResult;

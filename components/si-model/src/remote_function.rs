use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RemoteFunctionError {}

pub type RemoteFunctionResult<T> = Result<T, RemoteFunctionError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RemoteFunctionKind {
    Resolver,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFunctionRequest {
    pub kind: RemoteFunctionKind,
    pub code: String,
    pub container_image: String,
    pub container_tag: String,
}

pub mod code_generation;
pub mod qualification;
pub mod update_dependent_values;

use futures::Future;
use thiserror::Error;
use serde::{Deserialize, Serialize};

use crate::{
    AccessBuilder, DalContext, StandardModelError, Visibility,
    WsEventError,
};

#[derive(Error, Debug)]
pub enum JobError {
    #[error("component error: {0}")]
    Component(String),
    #[error("attribute value error: {0}")]
    AttributeValue(String),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("wsevent error: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("empty request")]
    EmptyRequest,
    #[error("invalid arg type: {0:?}")]
    InvalidArgType(serde_json::Value),
    #[error("invalid request: {0:?}")]
    InvalidRequest(serde_json::Error),
    #[error("execution failed: {0:?}")]
    Failure(String),
}

pub type JobResult<T, E = JobError> = Result<T, E>;

pub type JobFuture = Box<dyn Future<Output = JobResult<()>>>;

// TODO: explore typetag crate to simplify deserialization, doesn't help with the channel name tho

pub trait Job: std::fmt::Debug + Serialize {
    fn prepare<'a, 'b, 'c>(&self, ctx: &'a DalContext<'b, 'c>) -> JobFuture;
    fn run<'a, 'b, 'c>(&self, ctx: &'a DalContext<'b, 'c>) -> JobFuture;
    fn name(&self) -> &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobWrapper {
    pub job: String,
    pub access_builder: AccessBuilder,
    pub visibility: Visibility,
}

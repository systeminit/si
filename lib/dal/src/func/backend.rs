use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{label_list::ToLabelList, PropKind};

pub mod array;
pub mod boolean;
pub mod identity;
pub mod integer;
pub mod js_attribute;
pub mod js_code_generation;
pub mod js_qualification;
pub mod js_resource;
pub mod js_workflow;
pub mod map;
pub mod prop_object;
pub mod string;
pub mod validation;

#[derive(Error, Debug)]
pub enum FuncBackendError {
    #[error("invalid data - expected an array, got: {0}")]
    InvalidArrayData(serde_json::Value),
    #[error("invalid data - expected a valid array entry value, got: {0}")]
    InvalidArrayEntryData(serde_json::Value),
    #[error("invalid data - expected a boolean, got: {0}")]
    InvalidBooleanData(serde_json::Value),
    #[error("invalid data - expected an integer, got: {0}")]
    InvalidIntegerData(serde_json::Value),
    #[error("invalid data - expected a map, got: {0}")]
    InvalidMapData(serde_json::Value),
    #[error("invalid data - expected a prop object, got: {0}")]
    InvalidPropObjectData(serde_json::Value),
    #[error("invalid data - expected a string, got: {0}")]
    InvalidStringData(serde_json::Value),
    #[error("expected same array entry prop kinds - expected {0}, found: {1}")]
    DifferingArrayEntryPropKinds(PropKind, PropKind),
    #[error("result failure: kind={kind}, message={message}")]
    ResultFailure { kind: String, message: String },
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("invalid data - got unset when not expected")]
    UnexpectedUnset,
    #[error("veritech client error: {0}")]
    VeritechClient(#[from] veritech::ClientError),
}

pub type FuncBackendResult<T> = Result<T, FuncBackendError>;

#[derive(
    Deserialize,
    Serialize,
    Debug,
    Display,
    AsRefStr,
    PartialEq,
    Eq,
    EnumIter,
    EnumString,
    Clone,
    Copy,
)]
pub enum FuncBackendKind {
    Array,
    Boolean,
    /// Mathematical identity of the [`Func`](crate::Func)'s arguments.
    Identity,
    Integer,
    JsQualification,
    JsResourceSync,
    JsCodeGeneration,
    JsAttribute,
    JsWorkflow,
    JsCommand,
    Map,
    PropObject,
    String,
    Unset,
    // Commented out while we climb back up - Adam & Fletcher
    //Number (Float?),
    //EmptyObject,
    //EmptyArray,
    Json,
    //Js,
    ValidateStringValue,
}

#[derive(
    Deserialize,
    Serialize,
    Debug,
    Display,
    AsRefStr,
    PartialEq,
    Eq,
    EnumIter,
    EnumString,
    Clone,
    Copy,
)]
pub enum FuncBackendResponseType {
    Array,
    Boolean,
    /// Mathematical identity of the [`Func`](crate::Func)'s arguments.
    Identity,
    Integer,
    Map,
    PropObject,
    Qualification,
    ResourceSync,
    CodeGeneration,
    String,
    Unset,
    Json,
    Validation,
    Workflow,
    Command,
}

impl From<FuncBackendKind> for FuncBackendResponseType {
    fn from(backend_kind: FuncBackendKind) -> Self {
        match backend_kind {
            FuncBackendKind::Array => FuncBackendResponseType::Array,
            FuncBackendKind::Boolean => FuncBackendResponseType::Boolean,
            FuncBackendKind::Identity => FuncBackendResponseType::Identity,
            FuncBackendKind::Integer => FuncBackendResponseType::Integer,
            FuncBackendKind::JsQualification => FuncBackendResponseType::Qualification,
            FuncBackendKind::JsResourceSync => FuncBackendResponseType::ResourceSync,
            FuncBackendKind::JsCodeGeneration => FuncBackendResponseType::CodeGeneration,
            FuncBackendKind::JsAttribute => FuncBackendResponseType::String,
            FuncBackendKind::Map => FuncBackendResponseType::Map,
            FuncBackendKind::PropObject => FuncBackendResponseType::PropObject,
            FuncBackendKind::String => FuncBackendResponseType::String,
            FuncBackendKind::Unset => FuncBackendResponseType::Unset,
            FuncBackendKind::Json => FuncBackendResponseType::Json,
            FuncBackendKind::ValidateStringValue => FuncBackendResponseType::Validation,
        }
    }
}

impl ToLabelList for FuncBackendKind {}

use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{edit_field::ToSelectWidget, label_list::ToLabelList, PropKind};

pub mod array;
pub mod boolean;
pub mod integer;
pub mod js_qualification;
pub mod js_resource;
pub mod js_string;
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
    Integer,
    JsQualification,
    JsResourceSync,
    JsString,
    PropObject,
    String,
    Unset,
    // Commented out while we climb back up - Adam & Fletcher
    //Number (Float?),
    //EmptyObject,
    //EmptyArray,
    //Json,
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
    Integer,
    PropObject,
    Qualification,
    ResourceSync,
    String,
    Unset,
    // Commented out while we climb back up - Adam & Fletcher
    //Json,
    Validation,
}

impl ToLabelList for FuncBackendKind {}
impl ToSelectWidget for FuncBackendKind {}

pub mod validation;

use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use thiserror::Error;

use crate::{edit_field::ToSelectWidget, label_list::ToLabelList};

#[derive(Error, Debug)]
pub enum FuncBackendError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("invalid data - expected a string, got: {0}")]
    InvalidStringData(serde_json::Value),
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
    String,
    // Commented out while we climb back up - Adam & Fletcher
    //Number,
    //Boolean,
    //Object,
    //Array,
    //EmptyObject,
    //EmptyArray,
    Unset,
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
    String,
    // Commented out while we climb back up - Adam & Fletcher
    //Number,
    //Boolean,
    //Object,
    //Array,
    Unset,
    //Json,
    Validation,
}

impl ToLabelList for FuncBackendKind {}
impl ToSelectWidget for FuncBackendKind {}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendStringArgs {
    pub value: String,
}

impl FuncBackendStringArgs {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendString {
    args: FuncBackendStringArgs,
}

impl FuncBackendString {
    pub fn new(args: FuncBackendStringArgs) -> Self {
        Self { args }
    }

    pub fn response_type(&self) -> FuncBackendResponseType {
        FuncBackendResponseType::String
    }

    pub fn execute(&self) -> FuncBackendResult<serde_json::Value> {
        let value = serde_json::to_value(&self.args.value)?;
        // You can be damn sure this is a string, really - because
        // the inner type there is a string. But hey - better safe
        // than sorry!
        if !value.is_string() {
            return Err(FuncBackendError::InvalidStringData(value));
        }
        Ok(value)
    }
}

use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::func::binding_return_value::FuncBindingReturnValue;

#[derive(Error, Debug)]
pub enum QualificationError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("no value returned in qualification function result")]
    NoValue,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationErrorMessage {
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationResult {
    pub success: bool,
    pub errors: Vec<QualificationErrorMessage>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationView {
    pub message: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub result: Option<QualificationResult>,
}

impl TryFrom<FuncBindingReturnValue> for QualificationView {
    type Error = QualificationError;

    fn try_from(fbrv: FuncBindingReturnValue) -> Result<Self, Self::Error> {
        if let Some(qual_result_json) = fbrv.value() {
            let result = serde_json::from_value(qual_result_json.clone())?;
            Ok(QualificationView {
                message: "did it".to_string(),
                title: None,
                description: None,
                link: None,
                result: Some(result),
            })
        } else {
            Err(QualificationError::NoValue)
        }
    }
}

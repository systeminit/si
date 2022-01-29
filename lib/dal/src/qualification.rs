use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    func::{backend::validation::ValidationError, binding_return_value::FuncBindingReturnValue},
    Prop, QualificationPrototype,
};

use veritech::QualificationSubCheck;

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
    pub title: Option<String>,
    pub link: Option<String>,
    pub sub_checks: Option<Vec<QualificationSubCheck>>,
    // Pretty sure this field is no longer relevant.
    pub errors: Vec<QualificationErrorMessage>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationView {
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub sub_checks: Option<Vec<QualificationSubCheck>>,
    pub result: Option<QualificationResult>,
}

impl QualificationView {
    pub fn new_for_validation_errors(
        prop_validation_errors: Vec<(Prop, Vec<ValidationError>)>,
    ) -> QualificationView {
        let mut sub_checks: Vec<QualificationSubCheck> = Vec::new();
        let mut success = true;
        for (prop, validation_errors) in prop_validation_errors.into_iter() {
            for validation_error in validation_errors.into_iter() {
                let description =
                    format!("field {} failed: {}", prop.name(), validation_error.message);
                sub_checks.push(QualificationSubCheck {
                    description,
                    status: veritech::QualificationSubCheckStatus::Failure,
                });
                success = false;
            }
        }
        QualificationView {
            title: "All fields are valid".into(),
            description: None,
            link: None,
            sub_checks: None,
            result: Some(QualificationResult {
                success,
                title: None,
                link: None,
                sub_checks: Some(sub_checks),
                errors: Vec::new(),
            }),
        }
    }

    pub fn new_for_qualification_prototype(prototype: QualificationPrototype) -> QualificationView {
        QualificationView {
            title: prototype.title().into(),
            description: None,
            link: None,
            sub_checks: None,
            result: None,
        }
    }
}

impl TryFrom<FuncBindingReturnValue> for QualificationView {
    type Error = QualificationError;

    fn try_from(fbrv: FuncBindingReturnValue) -> Result<Self, Self::Error> {
        if let Some(qual_result_json) = fbrv.value() {
            let result = serde_json::from_value(qual_result_json.clone())?;
            Ok(QualificationView {
                title: "Unknown (no title provided)".to_string(),
                description: None,
                link: None,
                sub_checks: None,
                result: Some(result),
            })
        } else {
            Err(QualificationError::NoValue)
        }
    }
}

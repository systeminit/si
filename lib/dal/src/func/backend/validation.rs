use crate::func::backend::{FuncBackend, FuncBackendResult};
use crate::validation::{ValidationError, ValidationErrorKind, ValidationKind};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendValidation {
    args: FuncBackendValidationArgs,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendValidationArgs {
    pub value: Option<String>,
    pub kind: ValidationKind,
}

impl FuncBackendValidationArgs {
    pub fn new(value: Option<String>, kind: ValidationKind) -> Self {
        Self { value, kind }
    }
}

#[async_trait]
impl FuncBackend for FuncBackendValidation {
    type Args = FuncBackendValidationArgs;

    fn new(args: FuncBackendValidationArgs) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let maybe_value = self.args.value.clone();
        let mut validation_errors = vec![];

        if let Some(value) = maybe_value {
            match self.args.kind {
                ValidationKind::StringEquals(expected) => {
                    if expected != value {
                        validation_errors.push(ValidationError {
                            message: format!(
                                "value ({value}) does not match expected ({expected})"
                            ),
                            kind: ValidationErrorKind::StringDoesNotEqual,
                            link: None,
                            level: None,
                        });
                    }
                }
                ValidationKind::StringHasPrefix(expected) => {
                    if !value.starts_with(&expected) {
                        validation_errors.push(ValidationError {
                            message: format!(
                                "value ({value}) does not contain prefix ({expected})"
                            ),
                            kind: ValidationErrorKind::StringDoesNotHavePrefix,
                            link: None,
                            level: None,
                        });
                    }
                }
                ValidationKind::StringInStringArray(expected, display_expected) => {
                    let valid = expected.iter().any(|e| e == &value);
                    if !valid {
                        let message = if display_expected {
                            format!("value ({value}) not found in list of expected values ({expected:?})")
                        } else {
                            format!("value ({value}) not found in list of expected values")
                        };
                        validation_errors.push(ValidationError {
                            message,
                            kind: ValidationErrorKind::StringNotInStringArray,
                            link: None,
                            level: None,
                        });
                    }
                }
            }
        } else {
            validation_errors.push(ValidationError {
                message: "value must be present".to_string(),
                kind: ValidationErrorKind::ValueMustBePresent,
                link: None,
                level: None,
            })
        }

        let value = serde_json::to_value(validation_errors)?;
        Ok((Some(value.clone()), Some(value)))
    }
}

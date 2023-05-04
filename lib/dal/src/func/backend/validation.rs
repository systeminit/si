use async_trait::async_trait;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;

use crate::func::backend::{FuncBackend, FuncBackendResult};
use crate::validation::{Validation, ValidationError, ValidationErrorKind};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendValidation {
    args: FuncBackendValidationArgs,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendValidationArgs {
    pub validation: Validation,
}

impl FuncBackendValidationArgs {
    pub fn new(validation: Validation) -> Self {
        Self { validation }
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
        let mut validation_errors = Vec::new();
        let value_must_be_present_error = ValidationError {
            message: "This field is required".to_owned(),
            kind: ValidationErrorKind::ValueMustBePresent,
            link: None,
            level: None,
        };

        let maybe_validation_error = match self.args.validation {
            Validation::IntegerIsBetweenTwoIntegers { value, lower_bound, upper_bound } => match value {
                Some(value) => match value > lower_bound && value < upper_bound {
                    true => None,
                    false => Some(ValidationError {
                        message: format!("value ({value}) is not in between lower ({lower_bound}) and upper ({upper_bound}) bounds"),
                        kind: ValidationErrorKind::IntegerNotInBetweenTwoIntegers,
                        link: None,
                        level: None,
                    }),
                },
                None => Some(value_must_be_present_error),
            },
            Validation::IntegerIsNotEmpty { value} => match value {
                Some(_value) => None,
                None => Some(value_must_be_present_error),
            },
            Validation::StringIsNotEmpty { value} => match value {
                Some(value) => match value.is_empty() {
                    true => Some(value_must_be_present_error),
                    false => None,
                },
                None => Some(value_must_be_present_error),
            },
            Validation::StringIsValidIpAddr { value } => match value {
                Some(value) => match IpAddr::from_str(&value) {
                    Ok(_) => None,
                    Err(e) => Some(ValidationError {
                        message: format!("value ({value}) is an invalid ip address: {e}"),
                        kind: ValidationErrorKind::InvalidIpAddr,
                        link: None,
                        level: None,
                    }),
                },
                None => Some(value_must_be_present_error),
            },
            Validation::StringIsHexColor { value } => match value {
                Some(value) => {
                    let re = Regex::new(r"^#[\dA-Fa-f]{6,8}$").unwrap();
                    if re.is_match(value.as_str()) {
                        None
                    } else {
                        Some(ValidationError {
                            message: format!("value ({value}) is not a valid hex string"),
                            kind: ValidationErrorKind::InvalidHexString,
                            link: None,
                            level: None,
                        })
                    }
                },
                None => None,
            },
            Validation::StringEquals { value, expected } => match value {
                Some(value) => match value == expected {
                    true => None,
                    false => Some(ValidationError {
                        message: format!("value ({value}) does not match expected ({expected})"),
                        kind: ValidationErrorKind::StringDoesNotEqual,
                        link: None,
                        level: None,
                    }),
                },
                None => Some(value_must_be_present_error),
            },
            Validation::StringHasPrefix { value, expected } => match value {
                Some(value) => match value.starts_with(&expected) {
                    true => None,
                    false => Some(ValidationError {
                        message: format!("value ({value}) does not contain prefix ({expected})"),
                        kind: ValidationErrorKind::StringDoesNotHavePrefix,
                        link: None,
                        level: None,
                    }),
                },
                None => Some(value_must_be_present_error),
            },
            Validation::StringInStringArray {
                value,
                expected,
                display_expected,
            } => match value {
                Some(value) => match expected.iter().any(|e| e == &value) {
                    true => None,
                    false => Some(ValidationError {
                        message: match display_expected {
                            true => format!("value ({value}) not found in list of expected values ({expected:?})"),
                            false => format!("value ({value}) not found in list of expected values")
                        },
                        kind: ValidationErrorKind::StringNotInStringArray,
                        link: None,
                        level: None,
                    })
                },
                None => Some(value_must_be_present_error),
            },
        };

        // NOTE(nick): currently, the "find status" query expects an array with non-null values
        // to be returned. Since we may add the ability to return multiple errors in the future,
        // we are keeping the same same as before (i.e. Vec<ValidationError>).
        if let Some(validation_error) = maybe_validation_error {
            validation_errors.push(validation_error);
        }

        let value = serde_json::to_value(validation_errors)?;
        Ok((Some(value.clone()), Some(value)))
    }
}

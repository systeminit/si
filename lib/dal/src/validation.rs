//! This module is the starting point for the "validations" domain. A "validation" is a check
//! performed on a single "field" (abstractly defined here, usually corresponding to a
//! [`Prop`](crate::Prop) to ensure its shape is valid.
//!
//! A ["qualification"](crate::qualification) is a check performed on the shape formed by
//! multiple "fields" in a tree (abstractly defined here, usually corresponding to a
//! [`SchemaVariant`](crate::SchemaVariant).
//!
//! In instances where there is a dependency on information from more than that single "field",
//! then a ["qualification"](crate::qualification) is used instead of a "validation".

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{FuncId};

pub mod prototype;
// pub mod resolver;

/// Struct for creating a consumable error for the frontend when a "field" fails its validation
/// check.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub message: String,
    /// This really should be an enum at some point, but we need to figure out the set of values it
    /// should be first.
    pub level: Option<String>,
    pub kind: ValidationErrorKind,
    pub link: Option<String>,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValidationConstructorError {
    #[error("invalid value kind; expected {0}, but found {1}")]
    InvalidValueKind(&'static str, Value),
}

pub type ValidationConstructorResult<T> = Result<T, ValidationConstructorError>;

/// This enum represents everything needed to create a ["validation"](crate::validation). At
/// minimum, every variant has a field named "value" (or similar) to represent the incoming
/// value to "validate". When creating a validation for the first time, the "value" (or similar)
/// field is usually set to `None`.
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Validation {
    /// Validate that the "value" integer is between the lower and upper bound integers.
    IntegerIsBetweenTwoIntegers {
        value: Option<i64>,
        lower_bound: i64,
        upper_bound: i64,
    },
    /// Validate that the "value" integer is not empty
    IntegerIsNotEmpty { value: Option<i64> },
    /// Validate that the "value" string is the same as the expected string.
    StringEquals {
        value: Option<String>,
        expected: String,
    },
    /// Validate that the "value" string has the expected string as its prefix.
    StringHasPrefix {
        value: Option<String>,
        expected: String,
    },
    /// Validate that the "value" string exists in at least one place in the expected string array.
    StringInStringArray {
        value: Option<String>,
        expected: Vec<String>,
        /// This field toggles whether or not the expected array of strings should be displayed in
        /// the validation error message (if applicable).
        ///
        /// As an example: if the expected list contains over 100 items, it may be preferable to set
        /// this field to `false`. If the expect list contains 4 or 5 items, it may be preferable to
        /// set this field to `true`.
        display_expected: bool,
    },
    /// Validate that the "value" string is a Hex Color.
    StringIsHexColor { value: Option<String> },
    /// Validate that the "value" string is not empty
    StringIsNotEmpty { value: Option<String> },
    /// Validate that the "value" string is a valid [IpAddr](std::net::IpAddr).
    StringIsValidIpAddr { value: Option<String> },
}

impl Validation {
    /// Returns a new [`Validation`](Self) with the "value" (or similar) field mutated. The
    /// remaining fields' values will be identical.
    pub fn update_value(self, value: &Option<Value>) -> ValidationConstructorResult<Self> {
        let validation = match self {
            Validation::IntegerIsBetweenTwoIntegers {
                value: _,
                lower_bound,
                upper_bound,
            } => Validation::IntegerIsBetweenTwoIntegers {
                value: Self::value_as_i64(value)?,
                lower_bound,
                upper_bound,
            },
            Validation::IntegerIsNotEmpty { value: _ } => Validation::IntegerIsNotEmpty {
                value: Self::value_as_i64(value)?,
            },
            Validation::StringEquals { value: _, expected } => Validation::StringEquals {
                value: Self::value_as_string(value)?,
                expected,
            },
            Validation::StringHasPrefix { value: _, expected } => Validation::StringHasPrefix {
                value: Self::value_as_string(value)?,
                expected,
            },
            Validation::StringInStringArray {
                value: _,
                expected,
                display_expected,
            } => Validation::StringInStringArray {
                value: Self::value_as_string(value)?,
                expected,
                display_expected,
            },
            Validation::StringIsValidIpAddr { value: _ } => Validation::StringIsValidIpAddr {
                value: Self::value_as_string(value)?,
            },
            Validation::StringIsHexColor { value: _ } => Validation::StringIsHexColor {
                value: Self::value_as_string(value)?,
            },
            Validation::StringIsNotEmpty { value: _ } => Validation::StringIsNotEmpty {
                value: Self::value_as_string(value)?,
            },
        };
        Ok(validation)
    }

    fn value_as_string(maybe_value: &Option<Value>) -> ValidationConstructorResult<Option<String>> {
        match maybe_value {
            Some(value) => match value.as_str() {
                Some(success_value) => Ok(Some(success_value.to_string())),
                None => Err(ValidationConstructorError::InvalidValueKind(
                    "String",
                    value.clone(),
                )),
            },
            None => Ok(None),
        }
    }

    fn value_as_i64(maybe_value: &Option<Value>) -> ValidationConstructorResult<Option<i64>> {
        match maybe_value {
            Some(value) => match value.as_i64() {
                Some(success_value) => Ok(Some(success_value)),
                None => Err(ValidationConstructorError::InvalidValueKind(
                    "i64",
                    value.clone(),
                )),
            },
            None => Ok(None),
        }
    }
}

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    IntegerNotInBetweenTwoIntegers,
    InvalidHexString,
    InvalidIpAddr,
    JsValidation,
    StringDoesNotEqual,
    StringDoesNotHavePrefix,
    StringNotInStringArray,
    ValueMustBePresent,
}

impl ValidationErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IntegerNotInBetweenTwoIntegers => "IntegerNotInBetweenTwoIntegers",
            Self::InvalidHexString => "InvalidHexString",
            Self::InvalidIpAddr => "InvalidIpAddr",
            Self::StringDoesNotEqual => "StringDoesNotEqual",
            Self::StringDoesNotHavePrefix => "StringDoesNotHavePrefix",
            Self::StringNotInStringArray => "StringNotInStringArray",
            Self::ValueMustBePresent => "ValueMustBePresent",
            Self::JsValidation => "JsValidation",
        }
    }
}

#[remain::sorted]
#[derive(Debug)]
pub enum ValidationKind {
    Builtin(Validation),
    Custom(FuncId),
}

// pub async fn create_validation(
//     ctx: &DalContext,
//     validation_kind: ValidationKind,
//     builtin_func_id: FuncId,
//     prop_id: PropId,
//     schema_id: SchemaId,
//     schema_variant_id: SchemaVariantId,
// ) -> prototype::ValidationPrototypeResult<prototype::ValidationPrototype> {
//     let (validation_func_id, validation_args) = match validation_kind {
//         ValidationKind::Builtin(validation) => (
//             builtin_func_id,
//             serde_json::to_value(FuncBackendValidationArgs::new(validation))?,
//         ),

//         ValidationKind::Custom(func_id) => (func_id, serde_json::json!(null)),
//     };
//     let mut builder = prototype::context::ValidationPrototypeContext::builder();
//     builder
//         .set_prop_id(prop_id)
//         .set_schema_id(schema_id)
//         .set_schema_variant_id(schema_variant_id);

//     prototype::ValidationPrototype::new(
//         ctx,
//         validation_func_id,
//         validation_args,
//         builder.to_context(ctx).await?,
//     )
//     .await
// }

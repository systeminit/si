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

pub mod prototype;
pub mod resolver;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    /// This really should be an enum at some point, but we need to figure out the set of values it
    /// should be first.
    pub level: Option<String>,
    pub kind: ValidationErrorKind,
    pub link: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ValidationKind {
    /// Validate that the provided string equals the expected string (contained in this variant).
    StringEquals(String),
    /// Validate that the provided string has the expected string (contained in this variant) as
    /// its prefix.
    StringHasPrefix(String),
    /// Validate that the provided string exists in at least one place in the expected string array
    /// (contained in this variant). The second value in this variant is a boolean corresponding to
    /// whether or not the expected array of strings should be displayed. As an example: if the
    /// expected list contains over 100 items, it may be preferable to set the boolean to
    /// `false`. If the expect list contains 4 or 5 items, it may be preferable to set the boolean
    /// to `true`.
    StringInStringArray(Vec<String>, bool),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    StringDoesNotEqual,
    StringDoesNotHavePrefix,
    StringNotInStringArray,
    ValueMustBePresent,
}

impl ValidationErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StringDoesNotEqual => "StringDoesNotEqual",
            Self::StringDoesNotHavePrefix => "StringDoesNotHavePrefix",
            Self::StringNotInStringArray => "StringNotInStringArray",
            Self::ValueMustBePresent => "ValueMustBePresent",
        }
    }
}

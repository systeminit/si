use serde::{Deserialize, Serialize};

pub mod validate_string;
pub mod validate_string_array;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ValidationKind {
    ValidateString,
    ValidateStringArray,
}

impl ValidationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ValidateString => "validateString",
            Self::ValidateStringArray => "validateStringArray",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub message: String,
    /// This really should be an enum at some point, but we need to figure out the set of values it should be first.
    pub level: Option<String>,
    pub kind: ValidationKind,
    pub link: Option<String>,
}

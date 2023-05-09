use derive_builder::UninitializedFieldError;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use object_tree::Hash;

use super::SpecError;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ValidationSpec {
    IntegerIsBetweenTwoIntegers {
        lower_bound: i64,
        upper_bound: i64,
    },
    IntegerIsNotEmpty,
    StringEquals {
        expected: String,
    },
    StringHasPrefix {
        expected: String,
    },
    StringInStringArray {
        expected: Vec<String>,
        display_expected: bool,
    },
    StringIsValidIpAddr,
    StringIsHexColor,
    StringIsNotEmpty,
    CustomValidation {
        func_unique_id: Hash,
    },
}

impl ValidationSpec {
    pub fn builder() -> ValidationSpecBuilder {
        ValidationSpecBuilder::default()
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, PartialEq, EnumIter, EnumString, Display, Serialize, Deserialize,
)]
pub enum ValidationSpecKind {
    IntegerIsBetweenTwoIntegers,
    IntegerIsNotEmpty,
    StringEquals,
    StringHasPrefix,
    StringInStringArray,
    StringIsValidIpAddr,
    StringIsHexColor,
    StringIsNotEmpty,
    CustomValidation,
}

#[derive(Clone, Debug, Default)]
pub struct ValidationSpecBuilder {
    kind: Option<ValidationSpecKind>,
    upper_bound: Option<i64>,
    lower_bound: Option<i64>,
    expected_string: Option<String>,
    expected_string_array: Option<Vec<String>>,
    display_expected: Option<bool>,
    func_unique_id: Option<Hash>,
}

impl ValidationSpecBuilder {
    pub fn kind(&mut self, kind: ValidationSpecKind) -> &mut Self {
        self.kind = Some(kind);
        self
    }

    pub fn upper_bound(&mut self, upper_bound: i64) -> &mut Self {
        self.upper_bound = Some(upper_bound);
        self
    }

    pub fn lower_bound(&mut self, lower_bound: i64) -> &mut Self {
        self.lower_bound = Some(lower_bound);
        self
    }

    pub fn expected_string(&mut self, expected_string: String) -> &mut Self {
        self.expected_string = Some(expected_string);
        self
    }

    pub fn expected_string_array(&mut self, expected_string_array: Vec<String>) -> &mut Self {
        self.expected_string_array = Some(expected_string_array);
        self
    }

    pub fn display_expected(&mut self, display_expected: bool) -> &mut Self {
        self.display_expected = Some(display_expected);
        self
    }

    pub fn func_unique_id(&mut self, func_unique_id: Hash) -> &mut Self {
        self.func_unique_id = Some(func_unique_id);
        self
    }

    pub fn build(&self) -> Result<ValidationSpec, SpecError> {
        Ok(match self.kind {
            Some(kind) => match kind {
                ValidationSpecKind::IntegerIsBetweenTwoIntegers => {
                    ValidationSpec::IntegerIsBetweenTwoIntegers {
                        lower_bound: self
                            .lower_bound
                            .ok_or(UninitializedFieldError::from("lower_bound"))?,
                        upper_bound: self
                            .upper_bound
                            .ok_or(UninitializedFieldError::from("lower_bound"))?,
                    }
                }
                ValidationSpecKind::IntegerIsNotEmpty => ValidationSpec::IntegerIsNotEmpty,
                ValidationSpecKind::StringEquals => ValidationSpec::StringEquals {
                    expected: self
                        .expected_string
                        .as_ref()
                        .ok_or(UninitializedFieldError::from("expected_string"))?
                        .to_string(),
                },
                ValidationSpecKind::StringHasPrefix => ValidationSpec::StringHasPrefix {
                    expected: self
                        .expected_string
                        .as_ref()
                        .ok_or(UninitializedFieldError::from("expected_string"))?
                        .to_string(),
                },
                ValidationSpecKind::StringInStringArray => ValidationSpec::StringInStringArray {
                    display_expected: self
                        .display_expected
                        .ok_or(UninitializedFieldError::from("display_expected"))?,
                    expected: self
                        .expected_string_array
                        .clone()
                        .ok_or(UninitializedFieldError::from("expected_string"))?,
                },
                ValidationSpecKind::StringIsValidIpAddr => ValidationSpec::StringIsValidIpAddr,
                ValidationSpecKind::StringIsHexColor => ValidationSpec::StringIsHexColor,
                ValidationSpecKind::StringIsNotEmpty => ValidationSpec::StringIsNotEmpty,
                ValidationSpecKind::CustomValidation => ValidationSpec::CustomValidation {
                    func_unique_id: self
                        .func_unique_id
                        .ok_or(UninitializedFieldError::from("func_unique_id"))?,
                },
            },
            None => {
                return Err(UninitializedFieldError::from("kind").into());
            }
        })
    }
}

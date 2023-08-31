use derive_builder::UninitializedFieldError;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use super::SpecError;

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ValidationSpec {
    CustomValidation {
        #[serde(alias = "funcUniqueId")]
        func_unique_id: String,
        #[serde(alias = "uniqueId")]
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
    IntegerIsBetweenTwoIntegers {
        #[serde(alias = "lowerBound")]
        lower_bound: i64,
        #[serde(alias = "upperBound")]
        upper_bound: i64,
        #[serde(alias = "uniqueId")]
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
    IntegerIsNotEmpty {
        #[serde(alias = "uniqueId")]
        unique_id: Option<String>,
        deleted: bool,
    },
    StringEquals {
        expected: String,
        #[serde(alias = "uniqueId")]
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
    StringHasPrefix {
        expected: String,
        #[serde(alias = "uniqueId")]
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
    StringInStringArray {
        expected: Vec<String>,
        #[serde(alias = "displayExpected")]
        display_expected: bool,
        #[serde(alias = "uniqueId")]
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
    StringIsHexColor {
        #[serde(alias = "uniqueId")]
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
    StringIsNotEmpty {
        #[serde(alias = "uniqueId")]
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
    StringIsValidIpAddr {
        #[serde(alias = "uniqueId")]
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
}

impl ValidationSpec {
    pub fn builder() -> ValidationSpecBuilder {
        ValidationSpecBuilder::default()
    }
}

#[remain::sorted]
#[derive(
    Clone, Copy, Debug, Eq, Hash, PartialEq, EnumIter, EnumString, Display, Serialize, Deserialize,
)]
pub enum ValidationSpecKind {
    CustomValidation,
    IntegerIsBetweenTwoIntegers,
    IntegerIsNotEmpty,
    StringEquals,
    StringHasPrefix,
    StringInStringArray,
    StringIsHexColor,
    StringIsNotEmpty,
    StringIsValidIpAddr,
}

#[derive(Clone, Debug, Default)]
pub struct ValidationSpecBuilder {
    kind: Option<ValidationSpecKind>,
    upper_bound: Option<i64>,
    lower_bound: Option<i64>,
    expected_string: Option<String>,
    expected_string_array: Option<Vec<String>>,
    display_expected: Option<bool>,
    func_unique_id: Option<String>,
    unique_id: Option<String>,
    deleted: bool,
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

    pub fn func_unique_id(&mut self, func_unique_id: impl Into<String>) -> &mut Self {
        self.func_unique_id = Some(func_unique_id.into());
        self
    }

    pub fn unique_id(&mut self, unique_id: impl Into<String>) -> &mut Self {
        self.unique_id = Some(unique_id.into());
        self
    }

    pub fn deleted(&mut self, deleted: bool) -> &mut Self {
        self.deleted = deleted;
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
                        unique_id: self.unique_id.to_owned(),
                        deleted: self.deleted,
                    }
                }
                ValidationSpecKind::IntegerIsNotEmpty => ValidationSpec::IntegerIsNotEmpty {
                    unique_id: self.unique_id.to_owned(),
                    deleted: self.deleted,
                },
                ValidationSpecKind::StringEquals => ValidationSpec::StringEquals {
                    expected: self
                        .expected_string
                        .as_ref()
                        .ok_or(UninitializedFieldError::from("expected_string"))?
                        .to_string(),
                    unique_id: self.unique_id.to_owned(),
                    deleted: self.deleted,
                },
                ValidationSpecKind::StringHasPrefix => ValidationSpec::StringHasPrefix {
                    expected: self
                        .expected_string
                        .as_ref()
                        .ok_or(UninitializedFieldError::from("expected_string"))?
                        .to_string(),
                    unique_id: self.unique_id.to_owned(),
                    deleted: self.deleted,
                },
                ValidationSpecKind::StringInStringArray => ValidationSpec::StringInStringArray {
                    display_expected: self
                        .display_expected
                        .ok_or(UninitializedFieldError::from("display_expected"))?,
                    expected: self
                        .expected_string_array
                        .to_owned()
                        .ok_or(UninitializedFieldError::from("expected_string"))?,
                    unique_id: self.unique_id.to_owned(),
                    deleted: self.deleted,
                },
                ValidationSpecKind::StringIsValidIpAddr => ValidationSpec::StringIsValidIpAddr {
                    unique_id: self.unique_id.to_owned(),
                    deleted: self.deleted,
                },
                ValidationSpecKind::StringIsHexColor => ValidationSpec::StringIsHexColor {
                    unique_id: self.unique_id.to_owned(),
                    deleted: self.deleted,
                },
                ValidationSpecKind::StringIsNotEmpty => ValidationSpec::StringIsNotEmpty {
                    unique_id: self.unique_id.to_owned(),
                    deleted: self.deleted,
                },
                ValidationSpecKind::CustomValidation => ValidationSpec::CustomValidation {
                    func_unique_id: self
                        .func_unique_id
                        .to_owned()
                        .ok_or(UninitializedFieldError::from("func_unique_id"))?,
                    unique_id: self.unique_id.to_owned(),
                    deleted: self.deleted,
                },
            },
            None => {
                return Err(UninitializedFieldError::from("kind").into());
            }
        })
    }
}

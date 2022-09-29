use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::func::backend::validation::{ValidationError, ValidationKind};
use crate::func::backend::{FuncBackend, FuncBackendResult};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendValidateStringArrayValueArgs {
    pub value: Option<String>,
    pub expected: Vec<String>,
    /// Display the expected list. This is preferable to disable if the list has many items
    /// (e.g. 100+), but may be preferable to enable is the list has a few items (e.g. 4 or 5).
    pub display_expected: bool,
}

impl FuncBackendValidateStringArrayValueArgs {
    pub fn new(value: Option<String>, expected: Vec<String>, display_expected: bool) -> Self {
        Self {
            value,
            expected,
            display_expected,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendValidateStringArrayValue {
    args: FuncBackendValidateStringArrayValueArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendValidateStringArrayValue {
    type Args = FuncBackendValidateStringArrayValueArgs;

    fn new(args: FuncBackendValidateStringArrayValueArgs) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let maybe_value = self.args.value.clone();
        let expected = self.args.expected.clone();
        let mut validation_errors = vec![];

        if let Some(value) = maybe_value {
            let valid = expected.iter().any(|e| e == &value);
            if !valid {
                let message = if self.args.display_expected {
                    format!("value ({value}) not found in list of expected values ({expected:?})")
                } else {
                    format!("value ({value}) not found in list of expected values")
                };
                validation_errors.push(ValidationError {
                    message,
                    kind: ValidationKind::ValidateStringArray,
                    link: None,
                    level: None,
                });
            }
        } else {
            validation_errors.push(ValidationError {
                message: "value must be present".to_string(),
                kind: ValidationKind::ValidateStringArray,
                link: None,
                level: None,
            })
        }

        let value = serde_json::to_value(validation_errors)?;
        Ok((Some(value.clone()), Some(value)))
    }
}

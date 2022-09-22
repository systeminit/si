use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::func::backend::validation::{ValidationError, ValidationKind};
use crate::func::backend::{FuncBackend, FuncBackendResult};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendValidateStringValueArgs {
    pub value: Option<String>,
    pub expected: String,
}

impl FuncBackendValidateStringValueArgs {
    pub fn new(value: Option<String>, expected: String) -> Self {
        Self { value, expected }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendValidateStringValue {
    args: FuncBackendValidateStringValueArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendValidateStringValue {
    type Args = FuncBackendValidateStringValueArgs;

    fn new(args: FuncBackendValidateStringValueArgs) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let value = self.args.value.clone();
        let expected = self.args.expected.clone();
        let mut validation_errors = vec![];

        if let Some(v) = value {
            if v != expected {
                validation_errors.push(ValidationError {
                    message: format!("value ({v}) does not match expected ({expected})"),
                    kind: ValidationKind::ValidateString,
                    link: None,
                    level: None,
                });
            }
        } else {
            validation_errors.push(ValidationError {
                message: "value must be present".to_string(),
                kind: ValidationKind::ValidateString,
                link: None,
                level: None,
            })
        }

        let value = serde_json::to_value(validation_errors)?;
        Ok((Some(value.clone()), Some(value)))
    }
}

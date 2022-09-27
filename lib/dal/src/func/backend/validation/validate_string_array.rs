use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::func::backend::validation::{ValidationError, ValidationKind};
use crate::func::backend::{FuncBackend, FuncBackendResult};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendValidateStringArrayValueArgs {
    pub value: Option<String>,
    pub expected: Vec<String>,
}

impl FuncBackendValidateStringArrayValueArgs {
    pub fn new(value: Option<String>, expected: Vec<String>) -> Self {
        Self { value, expected }
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
                validation_errors.push(ValidationError {
                    message: format!("value ({value}) not found in list: {expected:?}"),
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

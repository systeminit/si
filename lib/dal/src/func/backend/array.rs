use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    func::backend::{FuncBackend, FuncBackendError, FuncBackendResult},
    PropKind,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendArrayArgs {
    pub value: Vec<Value>,
}

impl FuncBackendArrayArgs {
    pub fn new(value: Vec<Value>) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendArray {
    args: FuncBackendArrayArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendArray {
    type Args = FuncBackendArrayArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        // Ensure each entry of the array is valid and is of the same prop kind.
        let mut first_kind_found: Option<PropKind> = None;
        let value = serde_json::to_value(self.args.value.clone())?;
        for entry in &self.args.value {
            let entry_kind = if entry.is_array() {
                PropKind::Array
            } else if entry.is_i64() {
                PropKind::Integer
            } else if entry.is_f64() {
                PropKind::Float
            } else if entry.is_object() {
                PropKind::Object
            } else if entry.is_boolean() {
                PropKind::Boolean
            } else if entry.is_string() {
                PropKind::String
            } else {
                return Err(FuncBackendError::InvalidArrayEntryData(value));
            };

            if let Some(v) = &first_kind_found {
                if v != &entry_kind {
                    return Err(FuncBackendError::DifferingArrayEntryPropKinds(
                        *v, entry_kind,
                    ));
                }
            } else {
                first_kind_found = Some(entry_kind);
            }
        }
        Ok((Some(value), Some(serde_json::json!([]))))
    }
}

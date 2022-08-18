use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::func::backend::{FuncBackend, FuncBackendResult};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendPropObjectArgs {
    pub value: Map<String, Value>,
}

impl FuncBackendPropObjectArgs {
    pub fn new(value: Map<String, Value>) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendPropObject {
    args: FuncBackendPropObjectArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendPropObject {
    type Args = FuncBackendPropObjectArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let value = serde_json::Value::Object(self.args.value);
        Ok((Some(value), Some(serde_json::json!({}))))
    }
}

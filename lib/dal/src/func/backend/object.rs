use async_trait::async_trait;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    Map,
    Value,
};

use crate::func::backend::{
    FuncBackend,
    FuncBackendResult,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendObjectArgs {
    pub value: Map<String, Value>,
}

impl FuncBackendObjectArgs {
    pub fn new(value: Map<String, Value>) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendObject {
    args: FuncBackendObjectArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendObject {
    type Args = FuncBackendObjectArgs;

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

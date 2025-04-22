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
pub struct FuncBackendMapArgs {
    pub value: Map<String, Value>,
}

impl FuncBackendMapArgs {
    pub fn new(value: Map<String, Value>) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendMap {
    args: FuncBackendMapArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendMap {
    type Args = FuncBackendMapArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let value = serde_json::to_value(&self.args.value)?;
        Ok((Some(value), Some(serde_json::json!({}))))
    }
}

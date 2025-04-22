use async_trait::async_trait;
use serde::{
    Deserialize,
    Serialize,
};

use crate::func::backend::{
    FuncBackend,
    FuncBackendResult,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendStringArgs {
    pub value: String,
}

impl FuncBackendStringArgs {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendString {
    args: FuncBackendStringArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendString {
    type Args = FuncBackendStringArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let value = serde_json::to_value(&self.args.value)?;
        Ok((Some(value.clone()), Some(value)))
    }
}

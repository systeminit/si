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
pub struct FuncBackendIntegerArgs {
    pub value: i64,
}

impl FuncBackendIntegerArgs {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendInteger {
    args: FuncBackendIntegerArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendInteger {
    type Args = FuncBackendIntegerArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let value = serde_json::to_value(self.args.value)?;
        Ok((Some(value.clone()), Some(value)))
    }
}

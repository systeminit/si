use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::func::backend::{FuncBackend, FuncBackendResult};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendJsonArgs {
    pub value: serde_json::Value,
}

impl FuncBackendJsonArgs {
    pub fn new(value: serde_json::Value) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendJson {
    args: FuncBackendJsonArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendJson {
    type Args = FuncBackendJsonArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        Ok((Some(self.args.value.clone()), Some(self.args.value)))
    }
}

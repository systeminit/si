use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::func::backend::{FuncBackend, FuncBackendResult};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendFloatArgs {
    pub value: f64,
}

impl FuncBackendFloatArgs {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendFloat {
    args: FuncBackendFloatArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendFloat {
    type Args = FuncBackendFloatArgs;

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

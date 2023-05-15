use crate::func::backend::{FuncBackend, FuncBackendResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendDiffArgs {
    pub first: serde_json::Value,
    pub second: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendDiff {
    args: FuncBackendDiffArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendDiff {
    type Args = FuncBackendDiffArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let diff = serde_json::json!({
            "diff": self.args.first != self.args.second,
            "newValue": self.args.second
        });
        Ok((Some(diff.clone()), Some(diff)))
    }
}

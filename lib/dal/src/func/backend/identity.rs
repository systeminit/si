use crate::func::backend::{FuncBackend, FuncBackendResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendIdentityArgs {
    pub identity: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendIdentity {
    args: FuncBackendIdentityArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendIdentity {
    type Args = FuncBackendIdentityArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        Ok((Some(self.args.identity.clone()), Some(self.args.identity)))
    }
}

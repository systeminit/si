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
pub struct FuncBackendIdentityArgs {
    pub identity: Option<serde_json::Value>,
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
        let identity_val = serde_json::to_value(self.args.identity.clone())?;
        Ok((Some(identity_val.clone()), Some(identity_val)))
    }
}

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
pub struct FuncBackendNormalizeToArrayArgs {
    pub value: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendNormalizeToArray {
    args: FuncBackendNormalizeToArrayArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendNormalizeToArray {
    type Args = FuncBackendNormalizeToArrayArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        let value = match self.args.value {
            Some(payload) if payload.is_null() => serde_json::json!([]),
            Some(payload) if payload.is_array() => payload,
            Some(payload) => serde_json::to_value(vec![payload])?,
            None => serde_json::json!([]),
        };
        Ok((Some(value.clone()), Some(value)))
    }
}

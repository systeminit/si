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
pub struct FuncBackendResourcePayloadToValueArgs {
    pub payload: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendResourcePayloadToValue {
    args: FuncBackendResourcePayloadToValueArgs,
}

#[async_trait]
impl FuncBackend for FuncBackendResourcePayloadToValue {
    type Args = FuncBackendResourcePayloadToValueArgs;

    fn new(args: Self::Args) -> Box<Self> {
        Box::new(Self { args })
    }

    async fn inline(
        self: Box<Self>,
    ) -> FuncBackendResult<(Option<serde_json::Value>, Option<serde_json::Value>)> {
        // NOTE(nick): the func argument is of kind "object", which is a bit funky because the prop
        // kind that is exclusively used for this as an input is always of kind "string". However,
        // we will assume that the value is valid to become an object.
        let value = match self.args.payload {
            Some(payload) if payload.is_null() => serde_json::json!({}),
            Some(payload) => payload,
            None => serde_json::json!({}),
        };
        Ok((Some(value.clone()), Some(value)))
    }
}

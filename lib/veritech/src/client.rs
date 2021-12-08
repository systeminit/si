use futures::StreamExt;
use si_data::NatsClient;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{ResolverFunctionRequest, ResolverFunctionResult};

#[derive(Error, Debug)]
pub enum VeritechClientError {
    #[error("serde error")]
    SerdeJson(#[from] serde_json::Error),
    #[error("nats io error")]
    Nats(#[from] si_data::NatsError),
    #[error("no function result from cyclone; bug!")]
    NoResult,
}

pub type VeritechClientResult<T> = Result<T, VeritechClientError>;

#[instrument(name = "veritech.client.run_function", skip(nats, kind, code))]
pub async fn run_function(
    nats: &NatsClient,
    kind: impl Into<String>,
    code: impl Into<String>,
) -> VeritechClientResult<ResolverFunctionResult> {
    let kind = kind.into();
    let code = code.into();
    let request = ResolverFunctionRequest {
        kind,
        code,
        container_image: "foo".to_string(),
        container_tag: "latest".to_string(),
    };
    let mut reply_sub = nats
        .request_multi(
            "veritech.function.resolver",
            serde_json::to_string(&request)?,
        )
        .await?;

    let mut result: Option<ResolverFunctionResult> = None;
    // TODO - We will eventually want this to timeout if we don't receive the
    // payload in time. Lots of fanciness can ensue.
    while let Some(msg) = reply_sub.next().await.transpose()? {
        let json_payload: serde_json::Value = serde_json::from_slice(msg.data())?;
        // Then it is output
        if json_payload["stream"].is_null() {
            let function_result: ResolverFunctionResult = serde_json::from_value(json_payload)?;
            result = Some(function_result);
            break;
        }

        // TODO: We should do something here with output!
        dbg!(json_payload);
    }
    result.ok_or(VeritechClientError::NoResult)
}

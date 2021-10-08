use crate::{FunctionResult, ResolverFunctionRequest};
use si_data::NatsConn;
use thiserror::Error;
use tracing::instrument;

#[derive(Error, Debug)]
pub enum VeritechClientError {
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("no function result from cyclone; bug!")]
    NoResult,
}

pub type VeritechClientResult<T> = Result<T, VeritechClientError>;

#[instrument(name = "veritech.client.run_function", skip(nats, kind, code))]
pub async fn run_function(
    nats: &NatsConn,
    kind: impl Into<String>,
    code: impl Into<String>,
) -> VeritechClientResult<FunctionResult> {
    let kind = kind.into();
    let code = code.into();
    let request = ResolverFunctionRequest {
        kind,
        code,
        container_image: "foo".to_string(),
        container_tag: "latest".to_string(),
    };
    let reply_sub = nats
        .request_multi(
            "veritech.function.resolver",
            &serde_json::to_string(&request)?,
        )
        .await?;

    let mut result: Option<FunctionResult> = None;
    // TODO - We will eventually want this to timeout if we don't receive the
    // payload in time. Lots of fanciness can ensue.
    while let Some(msg) = reply_sub.next().await {
        let json_payload = serde_json::to_value(&msg.data)?;
        // Then it is output
        if json_payload["stream"].is_null() {
            dbg!(json_payload);
        } else {
            let function_result: FunctionResult = serde_json::from_value(json_payload)?;
            result = Some(function_result);
            break;
        }
    }
    result.ok_or(VeritechClientError::NoResult)
}

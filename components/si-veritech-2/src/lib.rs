pub mod telemetry;
use si_data::{nats::Message, NatsConn};
use si_model::RemoteFunctionRequest;
use si_settings::Settings;
use thiserror::Error;
use tracing::instrument;

#[derive(Error, Debug)]
pub enum VeritechError {
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type VeritechResult<T> = Result<T, VeritechError>;

pub async fn start(nats: NatsConn, settings: Settings) -> VeritechResult<()> {
    let sub = nats
        .queue_subscribe("veritech.function.>", "veritech_dispatch")
        .await?;
    while let Some(message) = sub.next().await {
        match process_message(nats.clone(), message).await {
            Ok(_) => {}
            Err(e) => tracing::error!("message processing failed: {}", e),
        }
    }
    sub.unsubscribe().await?;
    Ok(())
}

#[instrument(name = "veritech.process_message", skip(nats))]
pub async fn process_message(nats: NatsConn, message: Message) -> VeritechResult<()> {
    dbg!(&message);
    if message.reply.is_none() {
        tracing::error!("message is malformed; it must have a reply mailbox");
    }
    match message.subject.as_ref() {
        "veritech.function.resolver" => {
            run_resolver_function(nats, message).await?;
        }
        subject => {
            tracing::error!("unknown veritech path: {}", subject);
        }
    }
    Ok(())
}

#[instrument(name = "veritech.run_resolver_function", skip(nats))]
pub async fn run_resolver_function(nats: NatsConn, message: Message) -> VeritechResult<()> {
    let request: RemoteFunctionRequest = serde_json::from_slice(&message.data)?;
    Ok(())
}

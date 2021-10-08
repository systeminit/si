use futures::StreamExt;
use si_cyclone::{
    resolver_function::{ResolverFunctionExecutingMessage, ResolverFunctionRequest},
    Client,
};
use si_data::{nats::Message, NatsConn};
use si_settings::Settings;
use thiserror::Error;
use tracing::instrument;

#[derive(Error, Debug)]
pub enum VeritechServerError {
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("cyclone client error: {0}")]
    Cyclone(#[from] si_cyclone::client::ClientError),
    #[error("resolver function execution error: {0}")]
    ResolverFunction(#[from] si_cyclone::client::ResolverFunctionExecutionError),
    #[error("no return mailbox specified; bug!: {0:?}")]
    NoReturnMailbox(Message),
}

pub type VeritechServerResult<T> = Result<T, VeritechServerError>;

pub async fn start(nats: NatsConn, settings: Settings) -> VeritechServerResult<()> {
    dbg!("getting the fucking subscription asshole");
    let sub = nats.subscribe("veritech.function.resolver").await?;
    dbg!("got it and now waiting for messages");
    while let Some(message) = sub.next().await {
        dbg!(&message);
        dbg!(&message.data);
        match process_message(nats.clone(), message).await {
            Ok(_) => {}
            Err(e) => tracing::error!("message processing failed: {}", e),
        }
    }
    sub.unsubscribe().await?;
    Ok(())
}

#[instrument(name = "veritech.process_message", skip(nats))]
pub async fn process_message(nats: NatsConn, message: Message) -> VeritechServerResult<()> {
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
pub async fn run_resolver_function(nats: NatsConn, message: Message) -> VeritechServerResult<()> {
    let mailbox: &str = message
        .reply
        .as_deref()
        .ok_or_else(|| VeritechServerError::NoReturnMailbox(message.clone()))?;
    let request: ResolverFunctionRequest = serde_json::from_slice(&message.data)?;
    let mut cyclone = Client::http("127.0.0.1:5190")?;
    let mut progress = cyclone.execute_resolver(request).await?.start().await?;

    while let Some(msg) = progress.next().await {
        match msg {
            Ok(ResolverFunctionExecutingMessage::OutputStream(output)) => {
                nats.publish(mailbox, &serde_json::to_string(&output)?)
                    .await?;
            }
            Ok(unexpected) => todo!("deal with unexpected messages: {:?}", unexpected),
            Err(e) => todo!("deal with this: {:?}", e),
        }
    }
    let function_result = progress.finish().await?;
    nats.publish(mailbox, &serde_json::to_string(&function_result)?)
        .await?;

    Ok(())
}

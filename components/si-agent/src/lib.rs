use async_trait::async_trait;
use futures::StreamExt;
use si_transport::{AgentCommand, AgentCommandTopic, Topic};
use std::{borrow::Cow, collections::HashMap, sync::Arc};
use thiserror::Error;
use tracing::warn;

/// Quality of service level for topic subscriptions.
///
/// As an agent is essentially a consuming worker that is processing tasks which have potential
/// real world side effects (such as provisioning cloud resources, destroying deployments,
/// allocating switch ports, etc.), we only want *one* agent to process any given task *once*. That
/// is, a task for an agent is not assumed to be [idempotent].
///
/// Therefore, all Agent subscriptions will be `QoS::ExactlyOnce`. Make sense, dear reader?
///
/// [idempotent]: https://en.wikipedia.org/wiki/Idempotence
const SUBSCRIBE_QOS: QoS = QoS::ExactlyOnce;

pub mod spawn;

pub mod prelude {
    pub use crate::{Agent, DispatchBuilder, DispatchKey};
}

pub use si_transport::{
    AgentData, Header, Message, QoS, Transport, TypeHint, WireMessage, TEMP_AGENT_ID,
    TEMP_AGENT_INSTALLATION_ID,
};

pub type AgentResult<T> = std::result::Result<T, Error>;

#[async_trait]
pub trait Dispatch {
    async fn dispatch(&self, transport: &Transport, message: WireMessage) -> AgentResult<()>;
    fn dispatch_key(&self) -> DispatchKey;
}

pub trait Dispatchable: Dispatch + Sync + Send {}

pub trait DispatchBuilder {
    type Dispatchable: Dispatchable;

    fn new() -> Self;

    fn dispatch_key(&mut self, dispatch_key: DispatchKey) -> &mut Self;
    fn build(self) -> AgentResult<Self::Dispatchable>;

    fn integration_name(&self) -> &'static str;
    fn integration_service_name(&self) -> &'static str;
    fn object_type(&self) -> &'static str;
}

pub struct Agent {
    transport: Transport,
    dispatchers: HashMap<String, Arc<dyn Dispatchable>>,
}

impl Agent {
    pub fn builder<'a>(
        server_name: impl Into<Cow<'a, str>>,
        transport_server_uri: impl Into<String>,
        agent_id: impl Into<Cow<'a, str>>,
        agent_installation_id: impl Into<Cow<'a, str>>,
    ) -> AgentBuilder<'a> {
        AgentBuilder {
            server_name: server_name.into(),
            transport_server_uri: transport_server_uri.into(),
            agent_id: agent_id.into(),
            agent_installation_id: agent_installation_id.into(),
            dispatchers: HashMap::new(),
        }
    }

    pub async fn run(mut self) -> Result<(), Error> {
        let mut messages = self
            .transport
            .subscribe_to(self.subscriptions()?)
            .await?
            .messages();
        let transport = Arc::new(self.transport);

        while let Some(message) = messages.next().await {
            match self.dispatchers.get(message.topic_str()) {
                Some(dispatcher) => {
                    tokio::spawn(dispatch_message(
                        dispatcher.clone(),
                        transport.clone(),
                        message,
                    ));
                }
                None => {
                    warn!(
                        "no dispatcher to handle message from subscription, topic={}",
                        message.topic_str()
                    );
                    continue;
                }
            }
        }

        Ok(())
    }

    fn subscriptions(&self) -> Result<Vec<(Topic, QoS)>, Error> {
        let mut subscriptions = Vec::new();
        for topic_str in self.dispatchers.keys() {
            subscriptions.push((topic_str.parse()?, SUBSCRIBE_QOS));
        }

        Ok(subscriptions)
    }
}

pub struct AgentBuilder<'a> {
    server_name: Cow<'a, str>,
    transport_server_uri: String,
    agent_id: Cow<'a, str>,
    agent_installation_id: Cow<'a, str>,
    dispatchers: HashMap<DispatchKey, Arc<dyn Dispatchable>>,
}

impl<'a> AgentBuilder<'a> {
    pub fn dispatcher(&mut self, dispatcher: impl Dispatchable + 'static) -> &mut Self {
        self.dispatchers
            .insert(dispatcher.dispatch_key(), Arc::new(dispatcher));
        self
    }

    pub async fn build(self) -> Result<Agent, Error> {
        let transport = Transport::create(
            self.transport_server_uri,
            format!("agent:{}", self.server_name),
        )
        .await?;

        let mut dispatchers = HashMap::new();
        for (dispatch_key, dispatcher) in self.dispatchers.into_iter() {
            dispatchers.insert(
                AgentCommandTopic::builder()
                    .shared(true)
                    .agent_id(self.agent_id.as_ref())
                    .agent_installation_id(self.agent_installation_id.as_ref())
                    .integration_id(dispatch_key.integration_id)
                    .integration_service_id(dispatch_key.integration_service_id)
                    .object_type(dispatch_key.object_type)
                    .command(AgentCommand::Execute)
                    .build()
                    .to_string(),
                dispatcher,
            );
        }

        Ok(Agent {
            transport,
            dispatchers,
        })
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DispatchKey {
    integration_id: String,
    integration_service_id: String,
    object_type: String,
}

impl DispatchKey {
    pub fn new(
        integration_id: impl Into<String>,
        integration_service_id: impl Into<String>,
        object_type: impl Into<String>,
    ) -> Self {
        Self {
            integration_id: integration_id.into(),
            integration_service_id: integration_service_id.into(),
            object_type: object_type.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("execute error: {0}")]
    Execute(Box<dyn std::error::Error + Send + Sync>),
    #[error("invalid response topic type expected: {0}")]
    InvalidTopicType(Header),
    #[error("no dispatch function for action - integration service id: {0}, action name: {1}")]
    MissingDispatchFunction(String, String),
    #[error("missing dispatch key; this is a programmer error!")]
    MissingDispatchKey,
    #[error("missing response topic")]
    MissingResponseTopic,
    #[error("spawn command failed: {0}")]
    Spawn(#[from] spawn::Error),
    #[error("transport operation failed: {0}")]
    Transport(#[from] si_transport::Error),
}

impl Error {
    pub fn execute<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Execute(Box::new(err))
    }
}

async fn dispatch_message(
    dispatcher: Arc<dyn Dispatchable>,
    transport: Arc<Transport>,
    message: WireMessage,
) -> AgentResult<()> {
    dispatcher.dispatch(transport.as_ref(), message).await?;
    Ok(())
}

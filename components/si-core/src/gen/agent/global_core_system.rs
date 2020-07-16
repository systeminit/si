// Auto-generated code!
// No touchy!

use si_cea::EntityEvent;
use std::convert::TryInto;

pub struct GlobalCoreSystemDispatcherBuilder<T: GlobalCoreSystemDispatchFunctions> {
    dispatch_key: Option<si_agent::DispatchKey>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> si_agent::DispatchBuilder for GlobalCoreSystemDispatcherBuilder<T>
where
    T: GlobalCoreSystemDispatchFunctions + Sync + Send + 'static,
{
    type Dispatchable = GlobalCoreSystemDispatcher<T>;

    fn new() -> Self {
        Self {
            dispatch_key: None,
            _phantom: Default::default(),
        }
    }

    fn dispatch_key(&mut self, dispatch_key: si_agent::DispatchKey) -> &mut Self {
        self.dispatch_key = Some(dispatch_key);
        self
    }

    fn build(self) -> si_agent::AgentResult<Self::Dispatchable> {
        let dispatch_key = self
            .dispatch_key
            .ok_or(si_agent::Error::MissingDispatchKey)?;

        Ok(Self::Dispatchable::new(dispatch_key))
    }

    fn integration_name(&self) -> &'static str {
        "global"
    }

    fn integration_service_name(&self) -> &'static str {
        "core"
    }

    fn object_type(&self) -> &'static str {
        "system_entity_event"
    }
}

pub struct GlobalCoreSystemDispatcher<T: GlobalCoreSystemDispatchFunctions> {
    dispatch_key: si_agent::DispatchKey,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> GlobalCoreSystemDispatcher<T>
where
    T: GlobalCoreSystemDispatchFunctions,
{
    fn new(dispatch_key: si_agent::DispatchKey) -> Self {
        Self {
            dispatch_key,
            _phantom: Default::default(),
        }
    }

    async fn dispatch_event(
        transport: &si_agent::Transport,
        stream_header: si_agent::Header,
        entity_event: &mut crate::protobuf::SystemEntityEvent,
    ) -> si_agent::AgentResult<()> {
        match entity_event
            .action_name()
            .map_err(si_agent::Error::execute)?
        {
            "create" => T::create(transport, stream_header, entity_event).await,
            "sync" => T::sync(transport, stream_header, entity_event).await,
            invalid => Err(si_agent::Error::MissingDispatchFunction(
                entity_event
                    .integration_service_id()
                    .map_err(si_agent::Error::execute)?
                    .to_string(),
                invalid.to_string(),
            )),
        }
    }
}

#[async_trait::async_trait]
impl<T> si_agent::Dispatch for GlobalCoreSystemDispatcher<T>
where
    T: GlobalCoreSystemDispatchFunctions + Sync + Send,
{
    async fn dispatch(
        &self,
        transport: &si_agent::Transport,
        message: si_agent::WireMessage,
    ) -> si_agent::AgentResult<()> {
        let (_header, _qos, response_topic, mut entity_event) = {
            let msg: si_agent::Message<crate::protobuf::SystemEntityEvent> = message.try_into()?;
            msg.into_parts()
        };

        // Extract the response topic from the message, which must be a data header
        let mut response_topic =
            match response_topic.ok_or(si_agent::Error::MissingResponseTopic)? {
                si_agent::Header::AgentData(agent_data_topic) => agent_data_topic,
                topic => {
                    tracing::warn!(?topic, "response topic must be Header::AgentData type");
                    return Err(si_agent::Error::InvalidTopicType(topic));
                }
            };
        let stream_header: si_agent::Header = response_topic.clone().into();

        // Modify the response topic to determine the finalized topic
        response_topic.set_data(si_agent::AgentData::Finalize);
        let finalized_header = response_topic.into();

        si_cea::agent::EntityEventDispatch::prepare_entity_event(&mut entity_event)
            .map_err(si_agent::Error::execute)?;
        si_cea::agent::EntityEventDispatch::finish_entity_event(
            Self::dispatch_event(transport, stream_header.clone(), &mut entity_event)
                .await
                .map_err(si_cea::CeaError::action_error),
            transport,
            &mut entity_event,
            stream_header,
            finalized_header,
        )
        .await
        .map_err(si_agent::Error::execute)
    }

    fn dispatch_key(&self) -> si_agent::DispatchKey {
        self.dispatch_key.clone()
    }
}

impl<T> si_agent::Dispatchable for GlobalCoreSystemDispatcher<T> where
    T: GlobalCoreSystemDispatchFunctions + Sync + Send
{
}

#[async_trait::async_trait]
pub trait GlobalCoreSystemDispatchFunctions {
    async fn create(
        transport: &si_agent::Transport,
        stream_header: si_agent::Header,
        entity_event: &mut crate::protobuf::SystemEntityEvent,
    ) -> si_agent::AgentResult<()>;

    async fn sync(
        transport: &si_agent::Transport,
        stream_header: si_agent::Header,
        entity_event: &mut crate::protobuf::SystemEntityEvent,
    ) -> si_agent::AgentResult<()>;
}

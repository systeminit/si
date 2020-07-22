use si_cea::EntityEvent;

#[derive(Clone)]
pub struct GlobalCoreEdgeDispatcher<T: GlobalCoreEdgeDispatchFunctions> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: GlobalCoreEdgeDispatchFunctions> GlobalCoreEdgeDispatcher<T> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<T: GlobalCoreEdgeDispatchFunctions> si_cea::agent::dispatch::IntegrationActions
    for GlobalCoreEdgeDispatcher<T>
{
    fn integration_actions(&self) -> &'static [&'static str] {
        &["create", "sync"]
    }
}

impl<T: GlobalCoreEdgeDispatchFunctions> si_cea::agent::dispatch::IntegrationAndServiceName
    for GlobalCoreEdgeDispatcher<T>
{
    fn integration_name() -> &'static str {
        "global"
    }

    fn integration_service_name() -> &'static str {
        "core"
    }
}

#[async_trait::async_trait]
impl<T: GlobalCoreEdgeDispatchFunctions + Sync> si_cea::agent::dispatch::Dispatch
    for GlobalCoreEdgeDispatcher<T>
{
    type EntityEvent = T::EntityEvent;

    async fn dispatch(
        &self,
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()> {
        match entity_event.action_name()? {
            "create" => T::create(mqtt_client, entity_event).await,
            "sync" => T::sync(mqtt_client, entity_event).await,
            invalid => Err(si_cea::CeaError::DispatchFunctionMissing(
                entity_event.integration_service_id()?.to_string(),
                invalid.to_string(),
            )),
        }
    }
}

impl<T: GlobalCoreEdgeDispatchFunctions + Sync + Send + Clone>
    si_cea::agent::dispatch::IntegrationDispatch for GlobalCoreEdgeDispatcher<T>
{
}

#[async_trait::async_trait]
pub trait GlobalCoreEdgeDispatchFunctions {
    type EntityEvent: si_cea::EntityEvent + Send;

    async fn create(
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()>;

    async fn sync(
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()>;
}

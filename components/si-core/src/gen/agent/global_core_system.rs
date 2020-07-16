use si_cea::EntityEvent;

#[derive(Clone)]
pub struct GlobalCoreSystemDispatcher<T: GlobalCoreSystemDispatchFunctions> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: GlobalCoreSystemDispatchFunctions> GlobalCoreSystemDispatcher<T> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<T: GlobalCoreSystemDispatchFunctions> si_cea::agent::dispatch::IntegrationActions
    for GlobalCoreSystemDispatcher<T>
{
    fn integration_actions(&self) -> &'static [&'static str] {
        &["create", "sync"]
    }
}

impl<T: GlobalCoreSystemDispatchFunctions> si_cea::agent::dispatch::IntegrationAndServiceName
    for GlobalCoreSystemDispatcher<T>
{
    fn integration_name() -> &'static str {
        "global"
    }

    fn integration_service_name() -> &'static str {
        "core"
    }
}

#[async_trait::async_trait]
impl<T: GlobalCoreSystemDispatchFunctions + Sync> si_cea::agent::dispatch::Dispatch
    for GlobalCoreSystemDispatcher<T>
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

impl<T: GlobalCoreSystemDispatchFunctions + Sync + Send + Clone>
    si_cea::agent::dispatch::IntegrationDispatch for GlobalCoreSystemDispatcher<T>
{
}

#[async_trait::async_trait]
pub trait GlobalCoreSystemDispatchFunctions {
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

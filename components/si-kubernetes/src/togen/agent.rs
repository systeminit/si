use si_cea::agent::dispatch::codegen_prelude::*;

#[derive(Clone)]
pub struct AwsDispatcher<T: AwsDispatchFunctions> {
    _phantom: PhantomData<T>,
}

impl<T: AwsDispatchFunctions> AwsDispatcher<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }
}

impl<T: AwsDispatchFunctions> IntegrationActions for AwsDispatcher<T> {
    fn integration_actions(&self) -> &'static [&'static str] {
        &[
            "create",
            "sync",
            "edit_kubernetes_object",
            "edit_kubernetes_object_yaml",
        ]
    }
}

impl<T: AwsDispatchFunctions> IntegrationAndServiceName for AwsDispatcher<T> {
    fn integration_name() -> &'static str {
        T::integration_name()
    }

    fn integration_service_name() -> &'static str {
        T::integration_service_name()
    }
}

#[async_trait]
impl<T: AwsDispatchFunctions + Sync> Dispatch for AwsDispatcher<T> {
    type EntityEvent = T::EntityEvent;

    async fn dispatch(&self, mc: &MqttClient, ee: &mut Self::EntityEvent) -> CeaResult<()> {
        match ee.action_name() {
            "create" => T::create(mc, ee).await,
            "sync" => T::sync(mc, ee).await,
            "edit_kubernetes_object" => T::edit_kubernetes_object(mc, ee).await,
            "edit_kubernetes_object_yaml" => T::edit_kubernetes_object_yaml(mc, ee).await,
            invalid => Err(CeaError::DispatchFunctionMissing(
                ee.integration_service_id().to_string(),
                invalid.to_string(),
            )),
        }
    }
}

impl<T: AwsDispatchFunctions + Sync + Send + Clone> IntegrationDispatch for AwsDispatcher<T> {}

#[async_trait]
pub trait AwsDispatchFunctions: IntegrationAndServiceName {
    type EntityEvent: EntityEvent + Send;

    async fn create(
        mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()>;

    async fn sync(mqtt_client: &MqttClient, entity_event: &mut Self::EntityEvent) -> CeaResult<()>;

    async fn edit_kubernetes_object(
        mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()>;

    async fn edit_kubernetes_object_yaml(
        mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()>;
}

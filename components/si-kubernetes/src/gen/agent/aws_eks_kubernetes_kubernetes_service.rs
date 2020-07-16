use si_cea::EntityEvent;

#[derive(Clone)]
pub struct AwsEksKubernetesKubernetesServiceDispatcher<
    T: AwsEksKubernetesKubernetesServiceDispatchFunctions,
> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AwsEksKubernetesKubernetesServiceDispatchFunctions>
    AwsEksKubernetesKubernetesServiceDispatcher<T>
{
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<T: AwsEksKubernetesKubernetesServiceDispatchFunctions>
    si_cea::agent::dispatch::IntegrationActions for AwsEksKubernetesKubernetesServiceDispatcher<T>
{
    fn integration_actions(&self) -> &'static [&'static str] {
        &["create", "sync"]
    }
}

impl<T: AwsEksKubernetesKubernetesServiceDispatchFunctions>
    si_cea::agent::dispatch::IntegrationAndServiceName
    for AwsEksKubernetesKubernetesServiceDispatcher<T>
{
    fn integration_name() -> &'static str {
        "aws"
    }

    fn integration_service_name() -> &'static str {
        "eks_kubernetes"
    }
}

#[async_trait::async_trait]
impl<T: AwsEksKubernetesKubernetesServiceDispatchFunctions + Sync> si_cea::agent::dispatch::Dispatch
    for AwsEksKubernetesKubernetesServiceDispatcher<T>
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

impl<T: AwsEksKubernetesKubernetesServiceDispatchFunctions + Sync + Send + Clone>
    si_cea::agent::dispatch::IntegrationDispatch
    for AwsEksKubernetesKubernetesServiceDispatcher<T>
{
}

#[async_trait::async_trait]
pub trait AwsEksKubernetesKubernetesServiceDispatchFunctions {
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

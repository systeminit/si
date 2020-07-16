use si_cea::EntityEvent;

#[derive(Clone)]
pub struct AwsEksKubernetesKubernetesDeploymentDispatcher<
    T: AwsEksKubernetesKubernetesDeploymentDispatchFunctions,
> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AwsEksKubernetesKubernetesDeploymentDispatchFunctions>
    AwsEksKubernetesKubernetesDeploymentDispatcher<T>
{
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<T: AwsEksKubernetesKubernetesDeploymentDispatchFunctions>
    si_cea::agent::dispatch::IntegrationActions
    for AwsEksKubernetesKubernetesDeploymentDispatcher<T>
{
    fn integration_actions(&self) -> &'static [&'static str] {
        &["create", "apply", "sync"]
    }
}

impl<T: AwsEksKubernetesKubernetesDeploymentDispatchFunctions>
    si_cea::agent::dispatch::IntegrationAndServiceName
    for AwsEksKubernetesKubernetesDeploymentDispatcher<T>
{
    fn integration_name() -> &'static str {
        "aws"
    }

    fn integration_service_name() -> &'static str {
        "eks_kubernetes"
    }
}

#[async_trait::async_trait]
impl<T: AwsEksKubernetesKubernetesDeploymentDispatchFunctions + Sync>
    si_cea::agent::dispatch::Dispatch for AwsEksKubernetesKubernetesDeploymentDispatcher<T>
{
    type EntityEvent = T::EntityEvent;

    async fn dispatch(
        &self,
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()> {
        match entity_event.action_name()? {
            "create" => T::create(mqtt_client, entity_event).await,
            "apply" => T::apply(mqtt_client, entity_event).await,
            "sync" => T::sync(mqtt_client, entity_event).await,
            invalid => Err(si_cea::CeaError::DispatchFunctionMissing(
                entity_event.integration_service_id()?.to_string(),
                invalid.to_string(),
            )),
        }
    }
}

impl<T: AwsEksKubernetesKubernetesDeploymentDispatchFunctions + Sync + Send + Clone>
    si_cea::agent::dispatch::IntegrationDispatch
    for AwsEksKubernetesKubernetesDeploymentDispatcher<T>
{
}

#[async_trait::async_trait]
pub trait AwsEksKubernetesKubernetesDeploymentDispatchFunctions {
    type EntityEvent: si_cea::EntityEvent + Send;

    async fn create(
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()>;

    async fn apply(
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()>;

    async fn sync(
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()>;
}

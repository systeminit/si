use si_cea::EntityEvent;

#[derive(Clone)]
pub struct AwsEksKubernetesDeploymentDispatcher<T: AwsEksKubernetesDeploymentDispatchFunctions> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AwsEksKubernetesDeploymentDispatchFunctions> AwsEksKubernetesDeploymentDispatcher<T> {
    pub fn new() -> Self {
        Self {
            _phantom: Default::default(),
        }
    }
}

impl<T: AwsEksKubernetesDeploymentDispatchFunctions> si_cea::agent::dispatch::IntegrationActions
    for AwsEksKubernetesDeploymentDispatcher<T>
{
    fn integration_actions(&self) -> &'static [&'static str] {
        &[
            "create",
            "edit_kubernetes_object",
            "edit_kubernetes_object_yaml",
            "sync",
        ]
    }
}

impl<T: AwsEksKubernetesDeploymentDispatchFunctions>
    si_cea::agent::dispatch::IntegrationAndServiceName for AwsEksKubernetesDeploymentDispatcher<T>
{
    fn integration_name() -> &'static str {
        "aws"
    }

    fn integration_service_name() -> &'static str {
        "eks"
    }
}

#[async_trait::async_trait]
impl<T: AwsEksKubernetesDeploymentDispatchFunctions + Sync> si_cea::agent::dispatch::Dispatch
    for AwsEksKubernetesDeploymentDispatcher<T>
{
    type EntityEvent = T::EntityEvent;

    async fn dispatch(
        &self,
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()> {
        match entity_event.action_name()? {
            "create" => T::create(mqtt_client, entity_event).await,
            "edit_kubernetes_object" => T::edit_kubernetes_object(mqtt_client, entity_event).await,
            "edit_kubernetes_object_yaml" => {
                T::edit_kubernetes_object_yaml(mqtt_client, entity_event).await
            }
            "sync" => T::sync(mqtt_client, entity_event).await,
            invalid => Err(si_cea::CeaError::DispatchFunctionMissing(
                entity_event.integration_service_id()?.to_string(),
                invalid.to_string(),
            )),
        }
    }
}

impl<T: AwsEksKubernetesDeploymentDispatchFunctions + Sync + Send + Clone>
    si_cea::agent::dispatch::IntegrationDispatch for AwsEksKubernetesDeploymentDispatcher<T>
{
}

#[async_trait::async_trait]
pub trait AwsEksKubernetesDeploymentDispatchFunctions {
    type EntityEvent: si_cea::EntityEvent + Send;

    async fn create(
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()>;

    async fn edit_kubernetes_object(
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()>;

    async fn edit_kubernetes_object_yaml(
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()>;

    async fn sync(
        mqtt_client: &si_cea::MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> si_cea::CeaResult<()>;
}

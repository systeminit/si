use crate::agent::agent_apply;
use crate::gen::agent::{
    AwsEksKubernetesKubernetesDeploymentDispatchFunctions,
    AwsEksKubernetesKubernetesDeploymentDispatcher,
};
use crate::model::KubernetesDeploymentEntityEvent;
use crate::yaml_bytes;
use si_cea::agent::dispatch::prelude::*;

#[derive(Clone)]
pub struct AwsEksKubernetesKubernetesDeploymentDispatchFunctionsImpl;

#[async_trait]
impl AwsEksKubernetesKubernetesDeploymentDispatchFunctions
    for AwsEksKubernetesKubernetesDeploymentDispatchFunctionsImpl
{
    type EntityEvent = KubernetesDeploymentEntityEvent;

    async fn apply(
        mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { agent_apply(mqtt_client, entity_event, yaml_bytes!(entity_event)).await }
            .instrument(debug_span!("apply"))
            .await
    }

    async fn create(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("create")).await
    }

    async fn edit_kubernetes_object(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }
            .instrument(debug_span!("edit_kubernetes_object"))
            .await
    }

    async fn edit_kubernetes_object_yaml(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }
            .instrument(debug_span!("edit_kubernetes_object_yaml"))
            .await
    }

    async fn sync(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("sync")).await
    }
}

pub fn dispatcher() -> AwsEksKubernetesKubernetesDeploymentDispatcher<
    AwsEksKubernetesKubernetesDeploymentDispatchFunctionsImpl,
> {
    AwsEksKubernetesKubernetesDeploymentDispatcher::<
        AwsEksKubernetesKubernetesDeploymentDispatchFunctionsImpl,
    >::new()
}

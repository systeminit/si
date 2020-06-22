use crate::gen::agent::{
    AwsEksKubernetesKubernetesServiceDispatchFunctions, AwsEksKubernetesKubernetesServiceDispatcher,
};
use crate::model::KubernetesServiceEntityEvent;
use si_cea::agent::dispatch::prelude::*;

#[derive(Clone)]
pub struct AwsEksKubernetesKubernetesServiceDispatchFunctionsImpl;

#[async_trait]
impl AwsEksKubernetesKubernetesServiceDispatchFunctions
    for AwsEksKubernetesKubernetesServiceDispatchFunctionsImpl
{
    type EntityEvent = KubernetesServiceEntityEvent;

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

pub fn dispatcher() -> AwsEksKubernetesKubernetesServiceDispatcher<
    AwsEksKubernetesKubernetesServiceDispatchFunctionsImpl,
> {
    AwsEksKubernetesKubernetesServiceDispatcher::<
        AwsEksKubernetesKubernetesServiceDispatchFunctionsImpl,
    >::new()
}

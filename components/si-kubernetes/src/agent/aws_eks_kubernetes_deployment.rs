use crate::gen::agent::{
    AwsEksKubernetesKubernetesDeploymentDispatchFunctions,
    AwsEksKubernetesKubernetesDeploymentDispatcher,
};
use crate::model::KubernetesDeploymentEntityEvent;
use si_cea::agent::dispatch::prelude::*;

#[derive(Clone)]
pub struct AwsEksKubernetesKubernetesDeploymentDispatchFunctionsImpl;

#[async_trait]
impl AwsEksKubernetesKubernetesDeploymentDispatchFunctions
    for AwsEksKubernetesKubernetesDeploymentDispatchFunctionsImpl
{
    type EntityEvent = KubernetesDeploymentEntityEvent;

    async fn create(
        _mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Kubernetes like a motherfucker\n");
            entity_event.log(format!("{:?}", entity_event.input_entity()));
            entity_event.init_output_entity()?;
            Ok(())
        }
        .instrument(debug_span!("create"))
        .await
    }

    async fn edit_kubernetes_object(
        _mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Editing kubernetes_object like we just dont care\n");
            entity_event.init_output_entity()?;
            Ok(())
        }
        .instrument(debug_span!("edit_kubernetes_object"))
        .await
    }

    async fn edit_kubernetes_object_yaml(
        _mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Editing kubernetes_object_yaml like we just dont care\n");
            entity_event.init_output_entity()?;
            Ok(())
        }
        .instrument(debug_span!("edit_kubernetes_object_yaml"))
        .await
    }

    async fn sync(
        _mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Synchronizing like we just dont care\n");
            entity_event.init_output_entity()?;
            Ok(())
        }
        .instrument(debug_span!("sync"))
        .await
    }
}

pub fn dispatcher() -> AwsEksKubernetesKubernetesDeploymentDispatcher<
    AwsEksKubernetesKubernetesDeploymentDispatchFunctionsImpl,
> {
    AwsEksKubernetesKubernetesDeploymentDispatcher::<
        AwsEksKubernetesKubernetesDeploymentDispatchFunctionsImpl,
    >::new()
}

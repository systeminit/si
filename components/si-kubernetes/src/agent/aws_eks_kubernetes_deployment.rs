use crate::gen::agent::{
    AwsEksKubernetesKubernetesDeploymentDispatchFunctions,
    AwsEksKubernetesKubernetesDeploymentDispatcher,
};
use crate::kubectl::KubectlCommand;
use crate::model::KubernetesDeploymentEntityEvent;
use si_cea::agent::dispatch::prelude::*;
use si_cea::agent::utility::spawn_command::{spawn_command_with_stdin, CaptureOutput};
use std::env;

// TODO(fnichol): this should be entity/workspace/upstream info and not hardcoded
const NAMESPACE_VAR: &str = "KUBERNETES_NAMESPACE";
const NAMESPACE_DEFAULT: &str = "si";

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
        async {
            let cmd = KubectlCommand::new(namespace())
                .apply()
                .map_err(|err| CeaError::ActionError(err.to_string()))?;

            let stdin = entity_event
                .input_entity()?
                .properties()?
                .kubernetes_object_yaml
                .as_ref()
                .ok_or_else(|| si_data::required_field_err("kubernetes_object_yaml"))?
                .clone();

            spawn_command_with_stdin(
                mqtt_client,
                cmd,
                entity_event,
                CaptureOutput::None,
                Some(stdin),
            )
            .await?
            .success()?;

            Ok(())
        }
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

fn namespace() -> String {
    env::var(NAMESPACE_VAR).unwrap_or_else(|_| NAMESPACE_DEFAULT.to_string())
}

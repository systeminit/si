use crate::kubectl::KubectlCommand;
use si_cea::agent::utility::spawn_command::{spawn_command_with_stdin, CaptureOutput};
use si_cea::{CeaError, CeaResult, EntityEvent, MqttClient};
use std::env;

pub mod aws_eks_kubernetes_deployment;

// TODO(fnichol): this should be entity/workspace/upstream info and not hardcoded
const NAMESPACE_VAR: &str = "KUBERNETES_NAMESPACE";
const NAMESPACE_DEFAULT: &str = "si";

#[macro_export]
macro_rules! yaml_bytes {
    (
        $entity_event:expr $(,)?
    ) => {
        $crate::yaml_bytes!($entity_event, kubernetes_object_yaml)
    };
    (
        $entity_event:expr, $property:ident $(,)?
    ) => {
        $entity_event
            .input_entity()?
            .properties()?
            .$property
            .as_ref()
            .ok_or_else(|| si_data::required_field_err(stringify!($property)))?
            .clone()
    };
}

pub async fn agent_apply(
    mqtt_client: &MqttClient,
    entity_event: &mut impl EntityEvent,
    stdin_bytes: impl AsRef<[u8]>,
) -> CeaResult<()> {
    let cmd = KubectlCommand::new(namespace())
        .apply()
        .map_err(|err| CeaError::ActionError(err.to_string()))?;

    spawn_command_with_stdin(
        mqtt_client,
        cmd,
        entity_event,
        CaptureOutput::None,
        Some(stdin_bytes),
    )
    .await?
    .success()?;

    Ok(())
}

fn namespace() -> String {
    env::var(NAMESPACE_VAR).unwrap_or_else(|_| NAMESPACE_DEFAULT.to_string())
}

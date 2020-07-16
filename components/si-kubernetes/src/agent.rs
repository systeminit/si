use crate::kubectl::KubectlCommand;
use si_agent::{
    spawn::{spawn_command_with_stdin, CaptureOutput, OutputLine},
    AgentResult, Error, Header, Message, QoS, Transport,
};
use si_cea::EntityEvent;
use std::env;
use tokio::sync::mpsc;

pub mod aws_eks_kubernetes_kubernetes_deployment;
pub mod aws_eks_kubernetes_kubernetes_service;

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
            .input_entity()
            .map_err(si_agent::Error::execute)?
            .properties()
            .map_err(si_agent::Error::execute)?
            .$property
            .as_ref()
            .ok_or_else(|| si_data::required_field_err(stringify!($property)))
            .map_err(si_agent::Error::execute)?
            .clone()
            .into_bytes()
    };
}

pub async fn agent_apply(
    transport: &Transport,
    header: Header,
    entity_event: &mut impl EntityEvent,
    stdin_bytes: Vec<u8>,
) -> AgentResult<()> {
    let cmd = KubectlCommand::new(namespace())
        .apply()
        .map_err(Error::execute)?;

    let (tx, mut rx) = mpsc::channel(100000);

    let child = tokio::spawn(spawn_command_with_stdin(
        cmd,
        CaptureOutput::None,
        Some(stdin_bytes),
        tx,
    ));

    while let Some(output) = rx.recv().await {
        match output {
            OutputLine::Stdout(line) => {
                entity_event.log(line);
                // TODO(fnichol): send once, or twice or what?
                transport
                    .send(Message::new(
                        header.clone(),
                        QoS::AtMostOnce,
                        None::<Header>,
                        &entity_event,
                    ))
                    .await?;
            }
            OutputLine::Stderr(line) => {
                entity_event.error_log(line);
                // TODO(fnichol): send once, or twice or what?
                transport
                    .send(Message::new(
                        header.clone(),
                        QoS::AtMostOnce,
                        None::<Header>,
                        &entity_event,
                    ))
                    .await?;
            }
            OutputLine::Finished => {
                break;
            }
        }
    }

    child
        .await
        .map_err(si_agent::Error::execute)?
        .map_err(si_agent::Error::execute)?
        .success()?;

    Ok(())
}

fn namespace() -> String {
    env::var(NAMESPACE_VAR).unwrap_or_else(|_| NAMESPACE_DEFAULT.to_string())
}

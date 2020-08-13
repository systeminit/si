use si_transport::{AgentCommand, AgentData, Header, QoS, Transport, WireMessage};
use std::fmt::Debug;
use thiserror::Error;

/// Quality of service level for dispatching agent events.
///
/// As an agent is essentially a consuming worker that is processing tasks which have potential
/// real world side effects (such as provisioning cloud resources, destroying deployments,
/// allocating switch ports, etc.), we only want *one* agent to process any given task *once*. That
/// is, a task for an agent is not assumed to be [idempotent].
///
/// Therefore, all Agent event dispatches will be `QoS::ExactlyOnce`. Make sense, dear reader?
///
/// [idempotent]: https://en.wikipedia.org/wiki/Idempotence
const SEND_QOS: QoS = QoS::ExactlyOnce;

pub type Result<T> = std::result::Result<T, AgentExecuteSenderError>;

#[derive(Error, Debug)]
pub enum AgentExecuteSenderError {
    #[error("Missing a required field for the entity event: {0}")]
    MissingField(String),
    #[error("transport failure: {0}")]
    Transport(#[from] si_transport::Error),
}

#[derive(Debug, Clone)]
pub struct AgentExecuteSender {
    transport: Transport,
}

impl AgentExecuteSender {
    pub async fn create(
        client_name: impl AsRef<str>,
        server_uri: impl Into<String>,
    ) -> Result<Self> {
        let transport = Transport::create(
            server_uri,
            format!("agent_change_set_client:{}", client_name.as_ref()),
        )
        .await?;

        Ok(Self { transport })
    }

    pub async fn send(&mut self, entry: &serde_json::Value) -> Result<()> {
        let header = header_for_entry(entry)?;
        let qos = SEND_QOS;
        let response_header = Some(response_header_for_entry(entry)?);
        let payload_type = object_type_for_entry(entry)?;

        let message = WireMessage::from_parts(header, qos, response_header, payload_type, entry)?;

        self.transport.send(message).await?;

        Ok(())
    }
}

fn object_type_for_entry(entry: &serde_json::Value) -> Result<&str> {
    entry["siStorable"]["typeName"]
        .as_str()
        .ok_or(AgentExecuteSenderError::MissingField("typeName".into()))
}

fn header_for_entry(entry: &serde_json::Value) -> Result<Header> {
    let integration_id = entry["siProperties"]["integrationId"].as_str().ok_or(
        AgentExecuteSenderError::MissingField("integrationId".into()),
    )?;
    let integration_service_id = entry["siProperties"]["integrationServiceId"]
        .as_str()
        .ok_or(AgentExecuteSenderError::MissingField(
            "integrationServiceId".into(),
        ))?;
    let object_type = object_type_for_entry(entry)?;
    let id = entry["inputEntity"]["id"]
        .as_str()
        .ok_or(AgentExecuteSenderError::MissingField("inputEntity".into()))?;

    Ok(Header::new_command(
        si_transport::TEMP_AGENT_ID,
        si_transport::TEMP_AGENT_INSTALLATION_ID,
        integration_id,
        integration_service_id,
        object_type,
        id,
        AgentCommand::Execute,
    ))
}

fn response_header_for_entry(entry: &serde_json::Value) -> Result<Header> {
    let billing_account_id = entry["siProperties"]["billingAccountId"].as_str().ok_or(
        AgentExecuteSenderError::MissingField("billingAccountId".into()),
    )?;
    let organization_id = entry["siProperties"]["organizationId"].as_str().ok_or(
        AgentExecuteSenderError::MissingField("organizationId".into()),
    )?;
    let workspace_id = entry["siProperties"]["workspaceId"]
        .as_str()
        .ok_or(AgentExecuteSenderError::MissingField("workspaceId".into()))?;
    let integration_id = entry["siProperties"]["integrationId"].as_str().ok_or(
        AgentExecuteSenderError::MissingField("integrationId".into()),
    )?;
    let integration_service_id = entry["siProperties"]["integrationServiceId"]
        .as_str()
        .ok_or(AgentExecuteSenderError::MissingField(
            "integrationServiceId".into(),
        ))?;
    let object_type = object_type_for_entry(entry)?;
    let id = entry["inputEntity"]["id"]
        .as_str()
        .ok_or(AgentExecuteSenderError::MissingField("inputEntity".into()))?;

    Ok(Header::new_data(
        si_transport::TEMP_AGENT_ID,
        si_transport::TEMP_AGENT_INSTALLATION_ID,
        billing_account_id,
        organization_id,
        workspace_id,
        integration_id,
        integration_service_id,
        object_type,
        id,
        AgentData::Stream,
    ))
}

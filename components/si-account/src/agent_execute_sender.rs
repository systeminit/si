use si_transport::{AgentCommand, AgentData, Header, QoS, Transport, WireMessage};
use std::fmt::Debug;
use thiserror::Error;

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
        let topic = header_for_entry(entry)?.to_string();
        let qos = QoS::AtMostOnce;
        let response_topic = Some(response_topic_for_entry(entry)?.to_string());
        let payload_type = object_type_for_entry(entry)?;

        let message = WireMessage::from_parts(topic, qos, response_topic, payload_type, entry)?;

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

fn response_topic_for_entry(entry: &serde_json::Value) -> Result<Header> {
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

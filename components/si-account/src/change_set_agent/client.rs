use crate::change_set_agent::envelope::ChangeSetEnvelopeError;
use crate::change_set_agent::mqtt::{Message, MqttClient, PersistenceType};
use paho_mqtt::Error as MqttError;
use si_data::uuid_string;
use std::fmt::Debug;
use thiserror::Error;
use tracing::{debug, debug_span};
use tracing_futures::Instrument as _;

pub type Result<T> = std::result::Result<T, ChangeSetAgentClientError>;

#[derive(Error, Debug)]
pub enum ChangeSetAgentClientError {
    #[error("envelope payload failure: {0}")]
    Envelope(#[from] ChangeSetEnvelopeError),
    #[error("mqtt error: {0}")]
    Mqtt(#[from] MqttError),
    #[error("JSON serialization error: {0}")]
    JSON(#[from] serde_json::Error),
    #[error("Missing a required field for the entity event: {0}")]
    MissingField(String),
}

#[derive(Debug, Clone)]
pub struct ChangeSetAgentClient {
    pub mqtt: MqttClient,
}

impl ChangeSetAgentClient {
    pub async fn new(name: &str, vernemq_url: &str) -> Result<ChangeSetAgentClient> {
        // Create a client & define connect options
        let client_id = format!("agent_change_set_client:{}:{}", name, uuid_string());

        let mqtt = MqttClient::new()
            .server_uri(vernemq_url)
            .client_id(client_id)
            .persistence(PersistenceType::None)
            .create_client()?;
        mqtt.default_connect().await?;

        Ok(ChangeSetAgentClient { mqtt })
    }

    pub async fn dispatch(&self, change_set_entry: serde_json::Value) -> Result<()> {
        async {
            self.send(change_set_entry).await?;
            Ok(())
        }
        .instrument(debug_span!("change_set_agent_client"))
        .await
    }

    fn generate_topic(&self, change_set_entry: &serde_json::Value) -> Result<String> {
        let billing_account_id = change_set_entry["siProperties"]["billingAccountId"]
            .as_str()
            .ok_or(ChangeSetAgentClientError::MissingField(
                "billingAccountId".into(),
            ))?;

        let organization_id = change_set_entry["siProperties"]["organizationId"]
            .as_str()
            .ok_or(ChangeSetAgentClientError::MissingField(
                "organizationId".into(),
            ))?;

        let workspace_id = change_set_entry["siProperties"]["workspaceId"]
            .as_str()
            .ok_or(ChangeSetAgentClientError::MissingField(
                "workspaceId".into(),
            ))?;

        let integration_id = change_set_entry["siProperties"]["integrationId"]
            .as_str()
            .ok_or(ChangeSetAgentClientError::MissingField(
                "integrationId".into(),
            ))?;

        let integration_service_id = change_set_entry["siProperties"]["integrationServiceId"]
            .as_str()
            .ok_or(ChangeSetAgentClientError::MissingField(
                "integrationServiceId".into(),
            ))?;

        let entity_id = change_set_entry["inputEntity"]["id"].as_str().ok_or(
            ChangeSetAgentClientError::MissingField("inputEntity".into()),
        )?;

        let action_name = change_set_entry["actionName"]
            .as_str()
            .ok_or(ChangeSetAgentClientError::MissingField("actionName".into()))?;

        let id = change_set_entry["id"]
            .as_str()
            .ok_or(ChangeSetAgentClientError::MissingField("id".into()))?;

        let topic = format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}",
            billing_account_id,
            organization_id,
            workspace_id,
            integration_id,
            integration_service_id,
            entity_id,
            "action",
            action_name,
            id,
        );

        Ok(topic)
    }

    pub async fn send(&self, change_set_entry: serde_json::Value) -> Result<()> {
        async {
            let payload = serde_json::to_string(&change_set_entry)?;
            // We are very close to the broker - so no need to pretend that we are at
            // risk of not receiving our messages. Right?
            let topic = self.generate_topic(&change_set_entry)?;
            debug!(?topic, "topic");
            let msg = Message::new(topic, payload, 0);
            self.mqtt.publish(msg).await?;
            Ok(())
        }
        .instrument(debug_span!(
            "change_set_agent_client_send",
            ?change_set_entry
        ))
        .await
    }
}

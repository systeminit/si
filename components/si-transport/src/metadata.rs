use crate::error::Error;
use std::{fmt, str::FromStr};
use typed_builder::TypedBuilder;

const SHARED: &str = "$shared";
const CMD: &str = "cmd";
const DT: &str = "dt";
const AGENT: &str = "agent";

#[derive(Debug, Clone)]
pub enum Header {
    AgentCommand(AgentCommandHeader),
    AgentData(AgentDataHeader),
}

impl Header {
    pub fn new_command(
        agent_id: impl Into<String>,
        agent_installation_id: impl Into<String>,
        integration_id: impl Into<String>,
        integration_service_id: impl Into<String>,
        object_type: impl Into<String>,
        id: impl Into<String>,
        command: AgentCommand,
    ) -> Self {
        Self::AgentCommand(AgentCommandHeader {
            agent_id: agent_id.into(),
            agent_installation_id: agent_installation_id.into(),
            integration_id: integration_id.into(),
            integration_service_id: integration_service_id.into(),
            object_type: object_type.into(),
            id: id.into(),
            command,
        })
    }

    pub fn new_data(
        agent_id: impl Into<String>,
        agent_installation_id: impl Into<String>,
        billing_account_id: impl Into<String>,
        organization_id: impl Into<String>,
        workspace_id: impl Into<String>,
        integration_id: impl Into<String>,
        integration_service_id: impl Into<String>,
        object_type: impl Into<String>,
        id: impl Into<String>,
        data: AgentData,
    ) -> Self {
        Self::AgentData(AgentDataHeader {
            agent_id: agent_id.into(),
            agent_installation_id: agent_installation_id.into(),
            billing_account_id: billing_account_id.into(),
            organization_id: organization_id.into(),
            workspace_id: workspace_id.into(),
            integration_id: integration_id.into(),
            integration_service_id: integration_service_id.into(),
            object_type: object_type.into(),
            id: id.into(),
            data,
        })
    }

    pub fn into_topic(self) -> Topic {
        self.into()
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AgentCommand(header) => header.fmt(f),
            Self::AgentData(header) => header.fmt(f),
        }
    }
}

impl FromStr for Header {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(&format!("{}/{}", CMD, AGENT)) {
            Ok(Self::AgentCommand(AgentCommandHeader::from_str(s)?))
        } else if s.starts_with(&format!("{}/{}", DT, AGENT)) {
            Ok(Self::AgentData(AgentDataHeader::from_str(s)?))
        } else {
            Err(Error::InvalidHeaderOrTopic(s.to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentCommandHeader {
    agent_id: String,
    agent_installation_id: String,
    integration_id: String,
    integration_service_id: String,
    object_type: String,
    id: String,
    command: AgentCommand,
}

impl fmt::Display for AgentCommandHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = self.command.to_string();

        let parts = vec![
            CMD,
            AGENT,
            self.agent_id.as_ref(),
            self.agent_installation_id.as_ref(),
            self.integration_id.as_ref(),
            self.integration_service_id.as_ref(),
            self.object_type.as_ref(),
            self.id.as_ref(),
            command.as_ref(),
        ];

        f.write_str(&parts.join("/"))
    }
}

impl FromStr for AgentCommandHeader {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let topic = AgentCommandTopic::from_str(s)?;

        if topic.shared {
            return Err(Error::InvalidHeaderShared(s.to_string()));
        }

        let agent_id = topic
            .agent_id
            .ok_or_else(|| Error::missing_part("agent_id", s))?;
        if agent_id.is_empty() {
            return Err(Error::missing_part("agent_id", s));
        }
        let agent_installation_id = topic
            .agent_installation_id
            .ok_or_else(|| Error::missing_part("agent_installation_id", s))?;
        if agent_installation_id.is_empty() {
            return Err(Error::missing_part("agent_installation_id", s));
        }
        let integration_id = topic
            .integration_id
            .ok_or_else(|| Error::missing_part("integration_id", s))?;
        if integration_id.is_empty() {
            return Err(Error::missing_part("integration_id", s));
        }
        let integration_service_id = topic
            .integration_service_id
            .ok_or_else(|| Error::missing_part("integration_service_id", s))?;
        if integration_service_id.is_empty() {
            return Err(Error::missing_part("integration_service_id", s));
        }
        let object_type = topic
            .object_type
            .ok_or_else(|| Error::missing_part("object_type", s))?;
        if object_type.is_empty() {
            return Err(Error::missing_part("object_type", s));
        }
        let id = topic.id.ok_or_else(|| Error::missing_part("id", s))?;
        if id.is_empty() {
            return Err(Error::missing_part("id", s));
        }
        let command = topic
            .command
            .ok_or_else(|| Error::missing_part("command", s))?;

        Ok(Self {
            agent_id,
            agent_installation_id,
            integration_id,
            integration_service_id,
            object_type,
            id,
            command,
        })
    }
}

impl From<AgentCommandHeader> for Header {
    fn from(value: AgentCommandHeader) -> Self {
        Self::AgentCommand(value)
    }
}

#[derive(Debug, Clone)]
pub struct AgentDataHeader {
    agent_id: String,
    agent_installation_id: String,
    billing_account_id: String,
    organization_id: String,
    workspace_id: String,
    integration_id: String,
    integration_service_id: String,
    object_type: String,
    id: String,
    data: AgentData,
}

impl AgentDataHeader {
    pub fn set_data(&mut self, data: AgentData) {
        self.data = data;
    }
}

impl fmt::Display for AgentDataHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.to_string();

        let parts = vec![
            CMD,
            AGENT,
            self.agent_id.as_ref(),
            self.agent_installation_id.as_ref(),
            self.billing_account_id.as_ref(),
            self.organization_id.as_ref(),
            self.workspace_id.as_ref(),
            self.integration_id.as_ref(),
            self.integration_service_id.as_ref(),
            self.object_type.as_ref(),
            self.id.as_ref(),
            data.as_ref(),
        ];

        f.write_str(&parts.join("/"))
    }
}

impl FromStr for AgentDataHeader {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let topic = AgentDataTopic::from_str(s)?;

        if topic.shared {
            return Err(Error::InvalidHeaderShared(s.to_string()));
        }

        let agent_id = topic
            .agent_id
            .ok_or_else(|| Error::missing_part("agent_id", s))?;
        if agent_id.is_empty() {
            return Err(Error::missing_part("agent_id", s));
        }
        let agent_installation_id = topic
            .agent_installation_id
            .ok_or_else(|| Error::missing_part("agent_installation_id", s))?;
        if agent_installation_id.is_empty() {
            return Err(Error::missing_part("agent_installation_id", s));
        }
        let billing_account_id = topic
            .billing_account_id
            .ok_or_else(|| Error::missing_part("billing_account_id", s))?;
        if billing_account_id.is_empty() {
            return Err(Error::missing_part("billing_account_id", s));
        }
        let organization_id = topic
            .organization_id
            .ok_or_else(|| Error::missing_part("organization_id", s))?;
        if organization_id.is_empty() {
            return Err(Error::missing_part("organization_id", s));
        }
        let workspace_id = topic
            .workspace_id
            .ok_or_else(|| Error::missing_part("workspace_id", s))?;
        if workspace_id.is_empty() {
            return Err(Error::missing_part("workspace_id", s));
        }
        let integration_id = topic
            .integration_id
            .ok_or_else(|| Error::missing_part("integration_id", s))?;
        if integration_id.is_empty() {
            return Err(Error::missing_part("integration_id", s));
        }
        let integration_service_id = topic
            .integration_service_id
            .ok_or_else(|| Error::missing_part("integration_service_id", s))?;
        if integration_service_id.is_empty() {
            return Err(Error::missing_part("integration_service_id", s));
        }
        let object_type = topic
            .object_type
            .ok_or_else(|| Error::missing_part("object_type", s))?;
        if object_type.is_empty() {
            return Err(Error::missing_part("object_type", s));
        }
        let id = topic.id.ok_or_else(|| Error::missing_part("id", s))?;
        if id.is_empty() {
            return Err(Error::missing_part("id", s));
        }
        let data = topic.data.ok_or_else(|| Error::missing_part("data", s))?;

        Ok(Self {
            agent_id,
            agent_installation_id,
            billing_account_id,
            organization_id,
            workspace_id,
            integration_id,
            integration_service_id,
            object_type,
            id,
            data,
        })
    }
}

impl From<AgentDataHeader> for Header {
    fn from(value: AgentDataHeader) -> Self {
        Self::AgentData(value)
    }
}

#[derive(Debug, Clone)]
pub enum Topic {
    AgentCommand(AgentCommandTopic),
    AgentData(AgentDataTopic),
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AgentCommand(topic) => topic.fmt(f),
            Self::AgentData(topic) => topic.fmt(f),
        }
    }
}

impl FromStr for Topic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(&format!("{}/{}/{}", SHARED, CMD, AGENT))
            || s.starts_with(&format!("{}/{}", CMD, AGENT))
        {
            Ok(Self::AgentCommand(AgentCommandTopic::from_str(s)?))
        } else if s.starts_with(&format!("{}/{}/{}", SHARED, DT, AGENT))
            || s.starts_with(&format!("{}/{}", DT, AGENT))
        {
            Ok(Self::AgentData(AgentDataTopic::from_str(s)?))
        } else {
            Err(Error::InvalidHeaderOrTopic(s.to_string()))
        }
    }
}

impl From<Topic> for String {
    fn from(value: Topic) -> Self {
        match value {
            Topic::AgentCommand(value) => value.into(),
            Topic::AgentData(value) => value.into(),
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct AgentCommandTopic {
    #[builder(default = false)]
    shared: bool,
    #[builder(default, setter(strip_option, into))]
    agent_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    agent_installation_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    integration_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    integration_service_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    object_type: Option<String>,
    #[builder(default, setter(strip_option, into))]
    id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    command: Option<AgentCommand>,
}

impl fmt::Display for AgentCommandTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let command = self.command.as_ref().map(|c| c.to_string());

        let mut parts = vec![
            CMD,
            AGENT,
            self.agent_id.as_ref().map(String::as_str).unwrap_or("+"),
            self.agent_installation_id
                .as_ref()
                .map(String::as_str)
                .unwrap_or("+"),
            self.integration_id
                .as_ref()
                .map(String::as_str)
                .unwrap_or("+"),
            self.integration_service_id
                .as_ref()
                .map(String::as_str)
                .unwrap_or("+"),
            self.object_type.as_ref().map(String::as_str).unwrap_or("+"),
            self.id.as_ref().map(String::as_str).unwrap_or("+"),
            command.as_ref().map(String::as_str).unwrap_or("+"),
        ];
        if self.shared {
            parts.insert(0, SHARED);
        }

        f.write_str(&parts.join("/"))
    }
}

impl From<AgentCommandTopic> for String {
    fn from(value: AgentCommandTopic) -> Self {
        value.to_string()
    }
}

impl FromStr for AgentCommandTopic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('/');

        let mut current = parts
            .next()
            .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;
        let shared = if current == SHARED {
            current = parts
                .next()
                .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;
            true
        } else {
            false
        };
        if topic_part(current).as_ref().map(String::as_str) != Some(CMD) {
            return Err(Error::missing_part(CMD, s));
        }
        if next_topic_part(&mut parts, s)?.as_ref().map(String::as_str) != Some(AGENT) {
            return Err(Error::missing_part(AGENT, s));
        }

        let agent_id = next_topic_part(&mut parts, s)?;
        let agent_installation_id = next_topic_part(&mut parts, s)?;
        let integration_id = next_topic_part(&mut parts, s)?;
        let integration_service_id = next_topic_part(&mut parts, s)?;
        let object_type = next_topic_part(&mut parts, s)?;
        let id = next_topic_part(&mut parts, s)?;
        let command = match next_topic_part(&mut parts, s)? {
            Some(s) => Some(s.parse()?),
            None => None,
        };

        if parts.next().is_some() {
            return Err(Error::InvalidHeaderOrTopic(s.to_string()));
        }

        Ok(Self {
            shared,
            agent_id,
            agent_installation_id,
            integration_id,
            integration_service_id,
            object_type,
            id,
            command,
        })
    }
}

impl From<AgentCommandTopic> for Topic {
    fn from(value: AgentCommandTopic) -> Self {
        Self::AgentCommand(value)
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct AgentDataTopic {
    #[builder(default = false)]
    shared: bool,
    #[builder(default, setter(strip_option, into))]
    agent_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    agent_installation_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    billing_account_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    organization_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    workspace_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    integration_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    integration_service_id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    object_type: Option<String>,
    #[builder(default, setter(strip_option, into))]
    id: Option<String>,
    #[builder(default, setter(strip_option, into))]
    data: Option<AgentData>,
}

impl fmt::Display for AgentDataTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.as_ref().map(|c| c.to_string());

        let mut parts = vec![
            CMD,
            AGENT,
            self.agent_id.as_ref().map(String::as_str).unwrap_or("+"),
            self.agent_installation_id
                .as_ref()
                .map(String::as_str)
                .unwrap_or("+"),
            self.billing_account_id
                .as_ref()
                .map(String::as_str)
                .unwrap_or("+"),
            self.organization_id
                .as_ref()
                .map(String::as_str)
                .unwrap_or("+"),
            self.workspace_id
                .as_ref()
                .map(String::as_str)
                .unwrap_or("+"),
            self.integration_id
                .as_ref()
                .map(String::as_str)
                .unwrap_or("+"),
            self.integration_service_id
                .as_ref()
                .map(String::as_str)
                .unwrap_or("+"),
            self.object_type.as_ref().map(String::as_str).unwrap_or("+"),
            self.id.as_ref().map(String::as_str).unwrap_or("+"),
            data.as_ref().map(String::as_str).unwrap_or("+"),
        ];
        if self.shared {
            parts.insert(0, "$shared");
        }

        f.write_str(&parts.join("/"))
    }
}

impl From<AgentDataTopic> for String {
    fn from(value: AgentDataTopic) -> Self {
        value.to_string()
    }
}

impl FromStr for AgentDataTopic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('/');

        let mut current = parts
            .next()
            .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;
        let shared = if current == SHARED {
            current = parts
                .next()
                .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;
            true
        } else {
            false
        };
        if topic_part(current).as_ref().map(String::as_str) != Some(DT) {
            return Err(Error::missing_part(DT, s));
        }
        if next_topic_part(&mut parts, s)?.as_ref().map(String::as_str) != Some(AGENT) {
            return Err(Error::missing_part(AGENT, s));
        }

        let agent_id = next_topic_part(&mut parts, s)?;
        let agent_installation_id = next_topic_part(&mut parts, s)?;
        let billing_account_id = next_topic_part(&mut parts, s)?;
        let organization_id = next_topic_part(&mut parts, s)?;
        let workspace_id = next_topic_part(&mut parts, s)?;
        let integration_id = next_topic_part(&mut parts, s)?;
        let integration_service_id = next_topic_part(&mut parts, s)?;
        let object_type = next_topic_part(&mut parts, s)?;
        let id = next_topic_part(&mut parts, s)?;
        let data = match next_topic_part(&mut parts, s)? {
            Some(s) => Some(s.parse()?),
            None => None,
        };

        if parts.next().is_some() {
            return Err(Error::InvalidHeaderOrTopic(s.to_string()));
        }

        Ok(Self {
            shared,
            agent_id,
            agent_installation_id,
            billing_account_id,
            organization_id,
            workspace_id,
            integration_id,
            integration_service_id,
            object_type,
            id,
            data,
        })
    }
}

impl From<AgentDataTopic> for Topic {
    fn from(value: AgentDataTopic) -> Self {
        Self::AgentData(value)
    }
}

impl From<Header> for Topic {
    fn from(value: Header) -> Self {
        match value {
            Header::AgentCommand(value) => Self::AgentCommand(value.into()),
            Header::AgentData(value) => Self::AgentData(value.into()),
        }
    }
}

impl From<AgentCommandHeader> for AgentCommandTopic {
    fn from(value: AgentCommandHeader) -> Self {
        Self {
            shared: false,
            agent_id: Some(value.agent_id),
            agent_installation_id: Some(value.agent_installation_id),
            integration_id: Some(value.integration_id),
            integration_service_id: Some(value.integration_service_id),
            object_type: Some(value.object_type),
            id: Some(value.id),
            command: Some(value.command),
        }
    }
}

impl From<AgentDataHeader> for AgentDataTopic {
    fn from(value: AgentDataHeader) -> Self {
        Self {
            shared: false,
            agent_id: Some(value.agent_id),
            agent_installation_id: Some(value.agent_installation_id),
            billing_account_id: Some(value.billing_account_id),
            organization_id: Some(value.organization_id),
            workspace_id: Some(value.workspace_id),
            integration_id: Some(value.integration_id),
            integration_service_id: Some(value.integration_service_id),
            object_type: Some(value.object_type),
            id: Some(value.id),
            data: Some(value.data),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AgentCommand {
    Execute,
}

impl FromStr for AgentCommand {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "execute" => Ok(Self::Execute),
            invalid => Err(Error::InvalidAgentCommand(invalid.to_string())),
        }
    }
}

impl fmt::Display for AgentCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Execute => f.write_str("execute"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AgentData {
    Finalize,
    Status,
    Stream,
}

impl FromStr for AgentData {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "finalize" => Ok(Self::Finalize),
            "status" => Ok(Self::Status),
            "stream" => Ok(Self::Stream),
            invalid => Err(Error::InvalidAgentData(invalid.to_string())),
        }
    }
}

impl fmt::Display for AgentData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Finalize => f.write_str("finalize"),
            Self::Status => f.write_str("status"),
            Self::Stream => f.write_str("stream"),
        }
    }
}

fn topic_part(s: &str) -> Option<String> {
    match s {
        "+" | "" => None,
        s => Some(s.to_string()),
    }
}

fn next_topic_part(iter: &mut std::str::Split<'_, char>, s: &str) -> Result<Option<String>, Error> {
    let part = iter
        .next()
        .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;

    Ok(topic_part(part))
}

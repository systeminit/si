use crate::error::Error;
use std::{fmt, str::FromStr};
use typed_builder::TypedBuilder;

const SHARED: &str = "$share";
const CMD: &str = "cmd";
const DT: &str = "dt";
const AGENT: &str = "agent";
const WILDCARD: &str = "+";

#[derive(Debug, Clone, Hash, PartialEq)]
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

    pub fn satisfies(&self, topic: &Topic) -> bool {
        match (self, topic) {
            (Self::AgentCommand(header), Topic::AgentCommand(topic)) => header.satisfies(topic),
            (Self::AgentData(header), Topic::AgentData(topic)) => header.satisfies(topic),
            (_, _) => false,
        }
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

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct AgentCommandHeader {
    agent_id: String,
    agent_installation_id: String,
    integration_id: String,
    integration_service_id: String,
    object_type: String,
    id: String,
    command: AgentCommand,
}

impl AgentCommandHeader {
    pub fn satisfies(&self, topic: &AgentCommandTopic) -> bool {
        if topic_matches(&self.agent_id, &topic.agent_id)
            && topic_matches(&self.agent_installation_id, &topic.agent_installation_id)
            && topic_matches(&self.integration_id, &topic.integration_id)
            && topic_matches(&self.integration_service_id, &topic.integration_service_id)
            && topic_matches(&self.object_type, &topic.object_type)
            && topic_matches(&self.id, &topic.id)
        {
            if let Some(ref command) = topic.command {
                if command != &self.command {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
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

        if topic.shared_id.is_some() {
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

#[derive(Debug, Clone, Hash, PartialEq)]
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

    pub fn satisfies(&self, topic: &AgentDataTopic) -> bool {
        if topic_matches(&self.agent_id, &topic.agent_id)
            && topic_matches(&self.agent_installation_id, &topic.agent_installation_id)
            && topic_matches(&self.billing_account_id, &topic.billing_account_id)
            && topic_matches(&self.organization_id, &topic.organization_id)
            && topic_matches(&self.workspace_id, &topic.workspace_id)
            && topic_matches(&self.integration_id, &topic.integration_id)
            && topic_matches(&self.integration_service_id, &topic.integration_service_id)
            && topic_matches(&self.object_type, &topic.object_type)
            && topic_matches(&self.id, &topic.id)
        {
            if let Some(ref data) = topic.data {
                if data != &self.data {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
}

impl fmt::Display for AgentDataHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data.to_string();

        let parts = vec![
            DT,
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

        if topic.shared_id.is_some() {
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

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
        let mut parts = s.split('/');
        let mut current = parts.next();

        if let Some(SHARED) = current {
            current = parts.next();
            if current.is_none() {
                return Err(Error::InvalidHeaderOrTopic(s.to_string()));
            }
            current = parts.next();
        }

        match (current, parts.next()) {
            (Some(CMD), Some(AGENT)) => Ok(Self::AgentCommand(AgentCommandTopic::from_str(s)?)),
            (Some(DT), Some(AGENT)) => Ok(Self::AgentData(AgentDataTopic::from_str(s)?)),
            (_, _) => Err(Error::InvalidHeaderOrTopic(s.to_string())),
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

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd, TypedBuilder)]
pub struct AgentCommandTopic {
    #[builder(default, setter(strip_option, into))]
    shared_id: Option<String>,
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
        if let Some(ref shared_id) = self.shared_id {
            parts.insert(0, shared_id);
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
        let shared_id = if current == SHARED {
            let shared_id = parts
                .next()
                .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;
            current = parts
                .next()
                .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;
            Some(shared_id.to_string())
        } else {
            None
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
            shared_id,
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

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd, TypedBuilder)]
pub struct AgentDataTopic {
    #[builder(default, setter(strip_option, into))]
    shared_id: Option<String>,
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
            DT,
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
        if let Some(ref shared_id) = self.shared_id {
            parts.insert(0, shared_id);
            parts.insert(0, SHARED);
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
        let shared_id = if current == SHARED {
            let shared_id = parts
                .next()
                .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;
            current = parts
                .next()
                .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;
            Some(shared_id.to_string())
        } else {
            None
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
            shared_id,
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
            shared_id: None,
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
            shared_id: None,
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

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

fn topic_matches(header_part: &str, topic_part: &Option<String>) -> bool {
    match topic_part.as_ref().map(String::as_str) {
        Some(topic_part) => {
            if topic_part == WILDCARD || topic_part == header_part {
                true
            } else {
                false
            }
        }
        None => true,
    }
}

fn next_topic_part(iter: &mut std::str::Split<'_, char>, s: &str) -> Result<Option<String>, Error> {
    let part = iter
        .next()
        .ok_or_else(|| Error::InvalidHeaderOrTopic(s.to_string()))?;

    Ok(topic_part(part))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod header {
        use super::*;

        fn agent_command_header() -> Header {
            Header::new_command(
                "AID",
                "AIID",
                "IID",
                "ISID",
                "OTYPE",
                "ID",
                AgentCommand::Execute,
            )
        }

        fn agent_data_header() -> Header {
            Header::new_data(
                "AID",
                "AIID",
                "BAID",
                "OID",
                "WID",
                "IID",
                "ISID",
                "OTYPE",
                "ID",
                AgentData::Stream,
            )
        }

        fn agent_command_topic() -> Topic {
            AgentCommandTopic::builder()
                .shared_id("fans")
                .agent_id("AID")
                .agent_installation_id("AIID")
                .integration_id("IID")
                .integration_service_id("ISID")
                .object_type("OTYPE")
                .id("ID")
                .command(AgentCommand::Execute)
                .build()
                .into()
        }

        fn agent_data_topic() -> Topic {
            AgentDataTopic::builder()
                .shared_id("fans")
                .agent_id("AID")
                .agent_installation_id("AIID")
                .billing_account_id("BAID")
                .organization_id("OID")
                .workspace_id("WID")
                .integration_id("IID")
                .integration_service_id("ISID")
                .object_type("OTYPE")
                .id("ID")
                .data(AgentData::Stream)
                .build()
                .into()
        }

        #[test]
        fn display_agent_command() {
            let h = agent_command_header();

            assert_eq!(
                "cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute",
                h.to_string()
            );
        }

        #[test]
        fn display_agent_data() {
            let h = agent_data_header();

            assert_eq!(
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                h.to_string()
            );
        }

        #[test]
        fn from_str_agent_command() {
            let h = agent_command_header();

            assert_eq!(
                h,
                "cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute"
                    .parse()
                    .unwrap()
            );
        }

        #[test]
        fn satisfies_agent_command() {
            let h = agent_command_header();
            let t = agent_command_topic();

            assert!(h.satisfies(&t));
        }

        #[test]
        fn satisfies_agent_data() {
            let h = agent_data_header();
            let t = agent_data_topic();

            assert!(h.satisfies(&t));
        }

        #[test]
        fn satisfies_agent_command_against_agent_data_topic() {
            let h = agent_command_header();
            let t = agent_data_topic();

            assert_eq!(false, h.satisfies(&t));
        }

        #[test]
        fn satisfies_agent_data_against_agent_command_topic() {
            let h = agent_data_header();
            let t = agent_command_topic();

            assert_eq!(false, h.satisfies(&t));
        }
    }

    mod agent_command_header {
        use super::*;

        macro_rules! missing_parts {
            ($header_str:expr, $part:expr) => {
                match AgentCommandHeader::from_str($header_str) {
                    Err(Error::MissingHeaderOrTopicPart(p, s)) => {
                        assert_eq!(p, $part);
                        assert_eq!(s, $header_str);
                    }
                    Err(err) => panic!("wrong error type expected: {:?}", err),
                    Ok(_) => panic!("not expected to succeed"),
                }
            };
        }

        fn header() -> AgentCommandHeader {
            AgentCommandHeader {
                agent_id: "AID".to_string(),
                agent_installation_id: "AIID".to_string(),
                integration_id: "IID".to_string(),
                integration_service_id: "ISID".to_string(),
                object_type: "OTYPE".to_string(),
                id: "ID".to_string(),
                command: AgentCommand::Execute,
            }
        }

        fn topic() -> AgentCommandTopic {
            AgentCommandTopic::builder()
                .shared_id("fans")
                .agent_id("AID")
                .agent_installation_id("AIID")
                .integration_id("IID")
                .integration_service_id("ISID")
                .object_type("OTYPE")
                .id("ID")
                .command(AgentCommand::Execute)
                .build()
        }

        #[test]
        fn display() {
            let h = header();

            assert_eq!(
                "cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute",
                h.to_string()
            );
        }

        #[test]
        fn from_str() {
            assert_eq!(
                header(),
                "cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute"
                    .parse()
                    .unwrap()
            );
        }

        #[test]
        fn from_str_shared_invalid() {
            match AgentCommandHeader::from_str(
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute",
            ) {
                Err(Error::InvalidHeaderShared(_)) => assert!(true),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_no_agent_id() {
            missing_parts!("cmd/agent/+/AIID/IID/ISID/OTYPE/ID/execute", "agent_id");
            missing_parts!("cmd/agent//AIID/IID/ISID/OTYPE/ID/execute", "agent_id");
        }

        #[test]
        fn from_str_no_agent_installation_id() {
            missing_parts!(
                "cmd/agent/AID/+/IID/ISID/OTYPE/ID/execute",
                "agent_installation_id"
            );
            missing_parts!(
                "cmd/agent/AID//IID/ISID/OTYPE/ID/execute",
                "agent_installation_id"
            );
        }

        #[test]
        fn from_str_no_integration_id() {
            missing_parts!(
                "cmd/agent/AID/AIID/+/ISID/OTYPE/ID/execute",
                "integration_id"
            );
            missing_parts!(
                "cmd/agent/AID/AIID//ISID/OTYPE/ID/execute",
                "integration_id"
            );
        }

        #[test]
        fn from_str_no_integration_service_id() {
            missing_parts!(
                "cmd/agent/AID/AIID/IID/+/OTYPE/ID/execute",
                "integration_service_id"
            );
            missing_parts!(
                "cmd/agent/AID/AIID/IID//OTYPE/ID/execute",
                "integration_service_id"
            );
        }

        #[test]
        fn from_str_no_object_type() {
            missing_parts!("cmd/agent/AID/AIID/IID/ISID/+/ID/execute", "object_type");
            missing_parts!("cmd/agent/AID/AIID/IID/ISID//ID/execute", "object_type");
        }

        #[test]
        fn from_str_no_id() {
            missing_parts!("cmd/agent/AID/AIID/IID/ISID/OTYPE/+/execute", "id");
            missing_parts!("cmd/agent/AID/AIID/IID/ISID/OTYPE//execute", "id");
        }

        #[test]
        fn from_str_no_command() {
            missing_parts!("cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/+", "command");
            missing_parts!("cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/", "command");
        }

        #[test]
        fn satisfies() {
            let t = topic();

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_agent_id() {
            let mut t = topic();
            t.agent_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_agent_installation_id() {
            let mut t = topic();
            t.agent_installation_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_integration_id() {
            let mut t = topic();
            t.integration_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_integration_service_id() {
            let mut t = topic();
            t.integration_service_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_object_type() {
            let mut t = topic();
            t.object_type = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_id() {
            let mut t = topic();
            t.id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_command() {
            let mut t = topic();
            t.command = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_multiple() {
            let mut t = topic();
            t.agent_id = None;
            t.agent_installation_id = None;
            t.id = None;
            t.command = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_no_matches() {
            let mut t = topic();
            t.id = Some("nope".to_string());

            assert_eq!(false, header().satisfies(&t.into()));
        }
    }

    mod agent_data_header {
        use super::*;

        macro_rules! missing_parts {
            ($header_str:expr, $part:expr) => {
                match AgentDataHeader::from_str($header_str) {
                    Err(Error::MissingHeaderOrTopicPart(p, s)) => {
                        assert_eq!(p, $part);
                        assert_eq!(s, $header_str);
                    }
                    Err(err) => panic!("wrong error type expected: {:?}", err),
                    Ok(_) => panic!("not expected to succeed"),
                }
            };
        }

        fn header() -> AgentDataHeader {
            AgentDataHeader {
                agent_id: "AID".to_string(),
                agent_installation_id: "AIID".to_string(),
                billing_account_id: "BAID".to_string(),
                organization_id: "OID".to_string(),
                workspace_id: "WID".to_string(),
                integration_id: "IID".to_string(),
                integration_service_id: "ISID".to_string(),
                object_type: "OTYPE".to_string(),
                id: "ID".to_string(),
                data: AgentData::Stream,
            }
        }

        fn topic() -> AgentDataTopic {
            AgentDataTopic::builder()
                .shared_id("fans")
                .agent_id("AID")
                .agent_installation_id("AIID")
                .billing_account_id("BAID")
                .organization_id("OID")
                .workspace_id("WID")
                .integration_id("IID")
                .integration_service_id("ISID")
                .object_type("OTYPE")
                .id("ID")
                .data(AgentData::Stream)
                .build()
        }

        #[test]
        fn display() {
            let h = header();

            assert_eq!(
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                h.to_string()
            );
        }

        #[test]
        fn from_str() {
            assert_eq!(
                header(),
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream"
                    .parse()
                    .unwrap()
            );
        }

        #[test]
        fn from_str_shared_invalid() {
            match AgentDataHeader::from_str(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
            ) {
                Err(Error::InvalidHeaderShared(_)) => assert!(true),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_no_agent_id() {
            missing_parts!(
                "dt/agent/+/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                "agent_id"
            );
            missing_parts!(
                "dt/agent//AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                "agent_id"
            );
        }

        #[test]
        fn from_str_no_agent_installation_id() {
            missing_parts!(
                "dt/agent/AID/+/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                "agent_installation_id"
            );
            missing_parts!(
                "dt/agent/AID//BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                "agent_installation_id"
            );
        }

        #[test]
        fn from_str_shared_no_billing_account_id() {
            missing_parts!(
                "dt/agent/AID/AIID/+/OID/WID/IID/ISID/OTYPE/ID/stream",
                "billing_account_id"
            );
            missing_parts!(
                "dt/agent/AID/AIID//OID/WID/IID/ISID/OTYPE/ID/stream",
                "billing_account_id"
            );
        }

        #[test]
        fn from_str_shared_no_organization_id() {
            missing_parts!(
                "dt/agent/AID/AIID/BAID/+/WID/IID/ISID/OTYPE/ID/stream",
                "organization_id"
            );
            missing_parts!(
                "dt/agent/AID/AIID/BAID//WID/IID/ISID/OTYPE/ID/stream",
                "organization_id"
            );
        }

        #[test]
        fn from_str_shared_no_workspace_id() {
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID/+/IID/ISID/OTYPE/ID/stream",
                "workspace_id"
            );
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID//IID/ISID/OTYPE/ID/stream",
                "workspace_id"
            );
        }

        #[test]
        fn from_str_no_integration_id() {
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID/WID/+/ISID/OTYPE/ID/stream",
                "integration_id"
            );
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID/WID//ISID/OTYPE/ID/stream",
                "integration_id"
            );
        }

        #[test]
        fn from_str_no_integration_service_id() {
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID/WID/IID/+/OTYPE/ID/stream",
                "integration_service_id"
            );
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID/WID/IID//OTYPE/ID/stream",
                "integration_service_id"
            );
        }

        #[test]
        fn from_str_no_object_type() {
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/+/ID/stream",
                "object_type"
            );
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID//ID/stream",
                "object_type"
            );
        }

        #[test]
        fn from_str_no_id() {
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/+/stream",
                "id"
            );
            missing_parts!(
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE//stream",
                "id"
            );
        }

        #[test]
        fn from_str_no_data() {
            missing_parts!("dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/+", "data");
            missing_parts!("dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/", "data");
        }

        #[test]
        fn satisfies() {
            let t = topic();

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_agent_id() {
            let mut t = topic();
            t.agent_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_agent_installation_id() {
            let mut t = topic();
            t.agent_installation_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_billing_account_id() {
            let mut t = topic();
            t.billing_account_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_organization_id() {
            let mut t = topic();
            t.organization_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_workspace_id() {
            let mut t = topic();
            t.workspace_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_integration_id() {
            let mut t = topic();
            t.integration_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_integration_service_id() {
            let mut t = topic();
            t.integration_service_id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_object_type() {
            let mut t = topic();
            t.object_type = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_id() {
            let mut t = topic();
            t.id = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_data() {
            let mut t = topic();
            t.data = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_wildcard_multiple() {
            let mut t = topic();
            t.agent_id = None;
            t.agent_installation_id = None;
            t.id = None;
            t.data = None;

            assert!(header().satisfies(&t.into()));
        }

        #[test]
        fn satisfies_no_matches() {
            let mut t = topic();
            t.id = Some("nope".to_string());

            assert_eq!(false, header().satisfies(&t.into()));
        }
    }

    mod topic {
        use super::*;

        fn agent_command_topic(shared: bool) -> Topic {
            let builder = AgentCommandTopic::builder()
                .agent_id("AID")
                .agent_installation_id("AIID")
                .integration_id("IID")
                .integration_service_id("ISID")
                .object_type("OTYPE")
                .id("ID")
                .command(AgentCommand::Execute);

            Topic::AgentCommand(if shared {
                builder.shared_id("fans").build()
            } else {
                builder.build()
            })
        }

        fn agent_data_topic(shared: bool) -> Topic {
            let builder = AgentDataTopic::builder()
                .agent_id("AID")
                .agent_installation_id("AIID")
                .billing_account_id("BAID")
                .organization_id("OID")
                .workspace_id("WID")
                .integration_id("IID")
                .integration_service_id("ISID")
                .object_type("OTYPE")
                .id("ID")
                .data(AgentData::Stream);

            Topic::AgentData(if shared {
                builder.shared_id("fans").build()
            } else {
                builder.build()
            })
        }

        #[test]
        fn display_agent_command() {
            let t = agent_command_topic(true);

            assert_eq!(
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute",
                t.to_string(),
            );
        }

        #[test]
        fn display_agent_data() {
            let t = agent_data_topic(true);

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                t.to_string(),
            );
        }

        #[test]
        fn from_str_agent_command() {
            let t = agent_command_topic(false);

            assert_eq!(
                t,
                "cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_agent_command_shared() {
            let t = agent_command_topic(true);

            assert_eq!(
                t,
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_agent_data() {
            let t = agent_data_topic(false);

            assert_eq!(
                t,
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_agent_data_shared() {
            let t = agent_data_topic(true);

            assert_eq!(
                t,
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_invalid() {
            match Topic::from_str("$share/fans/ack/geez/nope") {
                Err(Error::InvalidHeaderOrTopic(_)) => assert!(true),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_agent_command_topic_for_string() {
            assert_eq!(
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute",
                String::from(agent_command_topic(true)),
            );
        }

        #[test]
        fn from_agent_data_topic_for_string() {
            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                String::from(agent_data_topic(true)),
            );
        }
    }

    mod agent_command_topic {
        use super::*;

        macro_rules! missing_parts {
            ($topic_str:expr) => {
                match AgentCommandTopic::from_str($topic_str) {
                    Err(Error::InvalidHeaderOrTopic(_)) => assert!(true),
                    Err(err) => panic!("wrong error type expected: {:?}", err),
                    Ok(_) => panic!("not expected to succeed"),
                }
            };
        }

        fn topic() -> AgentCommandTopic {
            AgentCommandTopic::builder()
                .shared_id("fans")
                .agent_id("AID")
                .agent_installation_id("AIID")
                .integration_id("IID")
                .integration_service_id("ISID")
                .object_type("OTYPE")
                .id("ID")
                .command(AgentCommand::Execute)
                .build()
        }

        #[test]
        fn defaults() {
            let t = AgentCommandTopic::builder().build();

            assert_eq!(None, t.shared_id);
            assert_eq!(None, t.agent_id);
            assert_eq!(None, t.agent_installation_id);
            assert_eq!(None, t.integration_id);
            assert_eq!(None, t.integration_service_id);
            assert_eq!(None, t.object_type);
            assert_eq!(None, t.id);
            assert_eq!(None, t.command);
        }

        #[test]
        fn display() {
            let t = topic();

            assert_eq!(
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute",
                t.to_string()
            );
        }

        #[test]
        fn display_no_shared() {
            let mut t = topic();
            t.shared_id = None;

            assert_eq!(
                "cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_agent_id() {
            let mut t = topic();
            t.agent_id = None;

            assert_eq!(
                "$share/fans/cmd/agent/+/AIID/IID/ISID/OTYPE/ID/execute",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_agent_installation_id() {
            let mut t = topic();
            t.agent_installation_id = None;

            assert_eq!(
                "$share/fans/cmd/agent/AID/+/IID/ISID/OTYPE/ID/execute",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_integration_id() {
            let mut t = topic();
            t.integration_id = None;

            assert_eq!(
                "$share/fans/cmd/agent/AID/AIID/+/ISID/OTYPE/ID/execute",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_integration_service_id() {
            let mut t = topic();
            t.integration_service_id = None;

            assert_eq!(
                "$share/fans/cmd/agent/AID/AIID/IID/+/OTYPE/ID/execute",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_object_type() {
            let mut t = topic();
            t.object_type = None;

            assert_eq!(
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/+/ID/execute",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_id() {
            let mut t = topic();
            t.id = None;

            assert_eq!(
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/+/execute",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_command() {
            let mut t = topic();
            t.command = None;

            assert_eq!(
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/+",
                t.to_string()
            );
        }

        #[test]
        fn from_string() {
            let s = "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute";

            assert_eq!(s, String::from(topic()));
            assert_eq!(s, Into::<String>::into(topic()));
        }

        #[test]
        fn from_str() {
            assert_eq!(
                topic(),
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_no_shared() {
            let mut t = topic();
            t.shared_id = None;

            assert_eq!(
                t,
                "cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_agent_id() {
            let mut t = topic();
            t.agent_id = None;

            assert_eq!(
                t,
                "$share/fans/cmd/agent/+/AIID/IID/ISID/OTYPE/ID/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_agent_installation_id() {
            let mut t = topic();
            t.agent_installation_id = None;

            assert_eq!(
                t,
                "$share/fans/cmd/agent/AID/+/IID/ISID/OTYPE/ID/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_integration_id() {
            let mut t = topic();
            t.integration_id = None;

            assert_eq!(
                t,
                "$share/fans/cmd/agent/AID/AIID/+/ISID/OTYPE/ID/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_integration_service_id() {
            let mut t = topic();
            t.integration_service_id = None;

            assert_eq!(
                t,
                "$share/fans/cmd/agent/AID/AIID/IID/+/OTYPE/ID/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_object_type() {
            let mut t = topic();
            t.object_type = None;

            assert_eq!(
                t,
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/+/ID/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_id() {
            let mut t = topic();
            t.id = None;

            assert_eq!(
                t,
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/+/execute"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_command() {
            let mut t = topic();
            t.command = None;

            assert_eq!(
                t,
                "$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID/+"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_shared_no_shared_id() {
            match AgentCommandTopic::from_str("$share") {
                Err(Error::InvalidHeaderOrTopic(a)) => assert_eq!(a, "$share"),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_shared_no_cmd() {
            match AgentCommandTopic::from_str("$share/wat") {
                Err(Error::InvalidHeaderOrTopic(_)) => assert!(true),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_no_cmd() {
            match AgentCommandTopic::from_str("wat") {
                Err(Error::MissingHeaderOrTopicPart(a, _)) => assert_eq!(a, "cmd"),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_no_agent() {
            match AgentCommandTopic::from_str("cmd/wat") {
                Err(Error::MissingHeaderOrTopicPart(a, _)) => assert_eq!(a, "agent"),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_no_agent_id() {
            missing_parts!("cmd/agent");
        }

        #[test]
        fn from_str_shared_no_agent_id() {
            missing_parts!("$share/fans/cmd/agent");
        }

        #[test]
        fn from_str_no_agent_installation_id() {
            missing_parts!("cmd/agent/AID");
        }

        #[test]
        fn from_str_shared_no_agent_installation_id() {
            missing_parts!("$share/fans/cmd/agent/AID");
        }

        #[test]
        fn from_str_no_integration_id() {
            missing_parts!("cmd/agent/AID/AIID");
        }

        #[test]
        fn from_str_shared_no_integration_id() {
            missing_parts!("$share/fans/cmd/agent/AID/AIID");
        }

        #[test]
        fn from_str_no_integration_service_id() {
            missing_parts!("cmd/agent/AID/AIID/IID");
        }

        #[test]
        fn from_str_no_shared_integration_service_id() {
            missing_parts!("$share/fans/cmd/agent/AID/AIID/IID");
        }

        #[test]
        fn from_str_no_object_type() {
            missing_parts!("cmd/agent/AID/AIID/IID/ISID");
        }

        #[test]
        fn from_str_shared_no_object_type() {
            missing_parts!("$share/fans/cmd/agent/AID/AIID/IID/ISID");
        }

        #[test]
        fn from_str_no_id() {
            missing_parts!("cmd/agent/AID/AIID/IID/ISID/OTYPE");
        }

        #[test]
        fn from_str_shared_no_id() {
            missing_parts!("$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE");
        }

        #[test]
        fn from_str_no_command() {
            missing_parts!("cmd/agent/AID/AIID/IID/ISID/OTYPE/ID");
        }

        #[test]
        fn from_str_shared_no_command() {
            missing_parts!("$share/fans/cmd/agent/AID/AIID/IID/ISID/OTYPE/ID");
        }
    }

    mod agent_data_topic {
        use super::*;

        macro_rules! missing_parts {
            ($topic_str:expr) => {
                match AgentDataTopic::from_str($topic_str) {
                    Err(Error::InvalidHeaderOrTopic(_)) => assert!(true),
                    Err(err) => panic!("wrong error type expected: {:?}", err),
                    Ok(_) => panic!("not expected to succeed"),
                }
            };
        }

        fn topic() -> AgentDataTopic {
            AgentDataTopic::builder()
                .shared_id("fans")
                .agent_id("AID")
                .agent_installation_id("AIID")
                .billing_account_id("BAID")
                .organization_id("OID")
                .workspace_id("WID")
                .integration_id("IID")
                .integration_service_id("ISID")
                .object_type("OTYPE")
                .id("ID")
                .data(AgentData::Stream)
                .build()
        }

        #[test]
        fn defaults() {
            let t = AgentDataTopic::builder().build();

            assert_eq!(None, t.shared_id);
            assert_eq!(None, t.agent_id);
            assert_eq!(None, t.agent_installation_id);
            assert_eq!(None, t.billing_account_id);
            assert_eq!(None, t.organization_id);
            assert_eq!(None, t.workspace_id);
            assert_eq!(None, t.integration_id);
            assert_eq!(None, t.integration_service_id);
            assert_eq!(None, t.object_type);
            assert_eq!(None, t.id);
            assert_eq!(None, t.data);
        }

        #[test]
        fn display() {
            let t = topic();

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_no_shared() {
            let mut t = topic();
            t.shared_id = None;

            assert_eq!(
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_agent_id() {
            let mut t = topic();
            t.agent_id = None;

            assert_eq!(
                "$share/fans/dt/agent/+/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_agent_installation_id() {
            let mut t = topic();
            t.agent_installation_id = None;

            assert_eq!(
                "$share/fans/dt/agent/AID/+/BAID/OID/WID/IID/ISID/OTYPE/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_billing_account_id() {
            let mut t = topic();
            t.billing_account_id = None;

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/+/OID/WID/IID/ISID/OTYPE/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_organization_id() {
            let mut t = topic();
            t.organization_id = None;

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/+/WID/IID/ISID/OTYPE/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_workspace_id() {
            let mut t = topic();
            t.workspace_id = None;

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/+/IID/ISID/OTYPE/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_integration_id() {
            let mut t = topic();
            t.integration_id = None;

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/+/ISID/OTYPE/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_integration_service_id() {
            let mut t = topic();
            t.integration_service_id = None;

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/+/OTYPE/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_object_type() {
            let mut t = topic();
            t.object_type = None;

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/+/ID/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_id() {
            let mut t = topic();
            t.id = None;

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/+/stream",
                t.to_string()
            );
        }

        #[test]
        fn display_wildcard_data() {
            let mut t = topic();
            t.data = None;

            assert_eq!(
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/+",
                t.to_string()
            );
        }

        #[test]
        fn from_string() {
            let s = "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream";

            assert_eq!(s, String::from(topic()));
            assert_eq!(s, Into::<String>::into(topic()));
        }

        #[test]
        fn from_str() {
            assert_eq!(
                topic(),
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_no_shared() {
            let mut t = topic();
            t.shared_id = None;

            assert_eq!(
                t,
                "dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_agent_id() {
            let mut t = topic();
            t.agent_id = None;

            assert_eq!(
                t,
                "$share/fans/dt/agent/+/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_agent_installation_id() {
            let mut t = topic();
            t.agent_installation_id = None;

            assert_eq!(
                t,
                "$share/fans/dt/agent/AID/+/BAID/OID/WID/IID/ISID/OTYPE/ID/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_integration_id() {
            let mut t = topic();
            t.integration_id = None;

            assert_eq!(
                t,
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/+/ISID/OTYPE/ID/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_integration_service_id() {
            let mut t = topic();
            t.integration_service_id = None;

            assert_eq!(
                t,
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/+/OTYPE/ID/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_object_type() {
            let mut t = topic();
            t.object_type = None;

            assert_eq!(
                t,
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/+/ID/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_id() {
            let mut t = topic();
            t.id = None;

            assert_eq!(
                t,
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/+/stream"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_wildcard_data() {
            let mut t = topic();
            t.data = None;

            assert_eq!(
                t,
                "$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID/+"
                    .parse()
                    .unwrap(),
            );
        }

        #[test]
        fn from_str_shared_no_shared_id() {
            match AgentDataTopic::from_str("$share") {
                Err(Error::InvalidHeaderOrTopic(a)) => assert_eq!(a, "$share"),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_shared_no_dt() {
            match AgentDataTopic::from_str("$share/wat") {
                Err(Error::InvalidHeaderOrTopic(_)) => assert!(true),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_no_dt() {
            match AgentDataTopic::from_str("wat") {
                Err(Error::MissingHeaderOrTopicPart(a, _)) => assert_eq!(a, "dt"),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_no_agent() {
            match AgentDataTopic::from_str("dt/wat") {
                Err(Error::MissingHeaderOrTopicPart(a, _)) => assert_eq!(a, "agent"),
                Err(err) => panic!("wrong error type expected: {:?}", err),
                Ok(_) => panic!("not expected to succeed"),
            }
        }

        #[test]
        fn from_str_no_agent_id() {
            missing_parts!("dt/agent");
        }

        #[test]
        fn from_str_shared_no_agent_id() {
            missing_parts!("$share/fans/dt/agent");
        }

        #[test]
        fn from_str_no_agent_installation_id() {
            missing_parts!("dt/agent/AID");
        }

        #[test]
        fn from_str_shared_no_agent_installation_id() {
            missing_parts!("$share/fans/dt/agent/AID");
        }

        #[test]
        fn from_str_no_billing_account_id() {
            missing_parts!("dt/agent/AID/AIID");
        }

        #[test]
        fn from_str_shared_no_billing_account_id() {
            missing_parts!("$share/fans/dt/agent/AID/AIID");
        }

        #[test]
        fn from_str_no_organization_id() {
            missing_parts!("dt/agent/AID/AIID/BAID");
        }

        #[test]
        fn from_str_shared_no_organization_id() {
            missing_parts!("$share/fans/dt/agent/AID/AIID/BAID");
        }

        #[test]
        fn from_str_no_workspace_id() {
            missing_parts!("dt/agent/AID/AIID/BAID/OID");
        }

        #[test]
        fn from_str_shared_no_workspace_id() {
            missing_parts!("$share/fans/dt/agent/AID/AIID/BAID/OID");
        }

        #[test]
        fn from_str_no_integration_id() {
            missing_parts!("dt/agent/AID/AIID/BAID/OID/WID");
        }

        #[test]
        fn from_str_shared_no_integration_id() {
            missing_parts!("$share/fans/dt/agent/AID/AIID/BAID/OID/WID");
        }

        #[test]
        fn from_str_no_integration_service_id() {
            missing_parts!("dt/agent/AID/AIID/BAID/OID/WID/IID");
        }

        #[test]
        fn from_str_no_shared_integration_service_id() {
            missing_parts!("$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID");
        }

        #[test]
        fn from_str_no_object_type() {
            missing_parts!("dt/agent/AID/AIID/BAID/OID/WID/IID/ISID");
        }

        #[test]
        fn from_str_shared_no_object_type() {
            missing_parts!("$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID");
        }

        #[test]
        fn from_str_no_id() {
            missing_parts!("dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE");
        }

        #[test]
        fn from_str_shared_no_id() {
            missing_parts!("$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE");
        }

        #[test]
        fn from_str_no_data() {
            missing_parts!("dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID");
        }

        #[test]
        fn from_str_shared_no_data() {
            missing_parts!("$share/fans/dt/agent/AID/AIID/BAID/OID/WID/IID/ISID/OTYPE/ID");
        }
    }

    mod agent_command {
        use super::*;

        #[test]
        fn display_execute() {
            assert_eq!("execute", AgentCommand::Execute.to_string());
        }

        #[test]
        fn from_str_execute() {
            assert_eq!(AgentCommand::Execute, "execute".parse().unwrap());
        }
    }

    mod agent_data {
        use super::*;

        #[test]
        fn display_finalize() {
            assert_eq!("finalize", AgentData::Finalize.to_string());
        }

        #[test]
        fn display_status() {
            assert_eq!("status", AgentData::Status.to_string());
        }

        #[test]
        fn display_stream() {
            assert_eq!("stream", AgentData::Stream.to_string());
        }

        #[test]
        fn from_str_finalize() {
            assert_eq!(AgentData::Finalize, "finalize".parse().unwrap());
        }

        #[test]
        fn from_str_status() {
            assert_eq!(AgentData::Status, "status".parse().unwrap());
        }

        #[test]
        fn from_str_stream() {
            assert_eq!(AgentData::Stream, "stream".parse().unwrap());
        }
    }
}

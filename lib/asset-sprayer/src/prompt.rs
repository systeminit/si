use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{AssetSprayerError, RawPromptYamlSource, Result};

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumDiscriminants,
)]
pub enum Prompt {
    AwsCliCommandPrompt(AwsCliCommandPromptKind, AwsCliCommand),
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    strum::AsRefStr,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    strum::IntoStaticStr,
    strum::VariantNames,
)]
pub enum AwsCliCommandPromptKind {
    AssetSchema,
    CreateAction,
    DestroyAction,
    RefreshAction,
    UpdateAction,
}

const AWS_CLI_DOCS_URL: &str = "https://docs.aws.amazon.com/cli/latest/reference";
const SI_DOCS_URL: &str =
    "https://raw.githubusercontent.com/systeminit/si/refs/heads/main/app/docs/src";

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AwsCliCommand(pub String, pub String);

impl Prompt {
    pub fn kind(&self) -> AwsCliCommandPromptKind {
        let Self::AwsCliCommandPrompt(kind, _) = self;
        *kind
    }

    pub async fn generate(&self, raw_prompt: &str) -> Result<CreateChatCompletionRequest> {
        let request = serde_yaml::from_str(raw_prompt)?;
        self.generate_request(request).await
    }

    async fn generate_request(
        &self,
        raw_request: CreateChatCompletionRequest,
    ) -> Result<CreateChatCompletionRequest> {
        let mut request = raw_request;
        for message in request.messages.iter_mut() {
            *message = self.generate_message(message.clone()).await?;
        }
        Ok(request)
    }

    async fn generate_message(
        &self,
        message: ChatCompletionRequestMessage,
    ) -> Result<ChatCompletionRequestMessage> {
        let mut message = message;
        match &mut message {
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(text),
                ..
            })
            | ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(text),
                ..
            }) => *text = self.generate_message_text(text).await?,
            _ => {}
        };
        Ok(message)
    }

    async fn generate_message_text(&self, text: &str) -> Result<String> {
        let text = match self {
            Self::AwsCliCommandPrompt(_, command) => text
                .replace("{AWS_COMMAND}", command.command())
                .replace("{AWS_SUBCOMMAND}", command.subcommand()),
        };
        let text = text
            .replace("{AWS_CLI_DOCS_URL}", AWS_CLI_DOCS_URL)
            .replace("{SI_DOCS_URL}", SI_DOCS_URL);
        self.fetch_message_text(&text).await
    }

    async fn fetch_message_text(&self, text: &str) -> Result<String> {
        // Fetch things between {FETCH} and {/FETCH}
        let mut result = String::new();
        let mut text = text;
        while let Some(fetch_start) = text.find("{FETCH}") {
            // Copy up to {FETCH}
            result.push_str(&text[..fetch_start]);
            text = &text[(fetch_start + "{FETCH}".len())..];

            if let Some(url_end) = text.find("{/FETCH}") {
                // Fetch the URL between {FETCH}...{/FETCH}
                result.push_str(&Self::get(&text[..url_end]).await?);
                text = &text[(url_end + "{/FETCH}".len())..];
            } else {
                return Err(AssetSprayerError::MissingEndFetch(self.clone()));
            }
        }

        // Copy the remainder of the text
        result.push_str(text);

        Ok(result)
    }

    async fn get(url: &str) -> Result<String> {
        if let Some(file_path) = url.strip_prefix("file:") {
            Ok(tokio::fs::read_to_string(file_path).await?)
        } else {
            info!("Fetching: {}", url);
            let client = reqwest::ClientBuilder::new()
                .user_agent("Wget/1.21.2")
                .build()?;
            let response = client.get(url).send().await?;
            Ok(response.error_for_status()?.text().await?)
        }
    }
}

impl AwsCliCommand {
    pub fn new(command: impl Into<String>, subcommand: impl Into<String>) -> Self {
        Self(command.into(), subcommand.into())
    }

    pub fn command(&self) -> &str {
        &self.0
    }

    pub fn subcommand(&self) -> &str {
        &self.1
    }
}

impl RawPromptYamlSource for Prompt {
    fn raw_prompt_yaml_relative_path(&self) -> &'static str {
        match self {
            Self::AwsCliCommandPrompt(kind, _) => kind.raw_prompt_yaml_relative_path(),
        }
    }

    fn raw_prompt_yaml_embedded(&self) -> &'static str {
        match self {
            Self::AwsCliCommandPrompt(kind, _) => kind.raw_prompt_yaml_embedded(),
        }
    }
}

impl RawPromptYamlSource for AwsCliCommandPromptKind {
    fn raw_prompt_yaml_relative_path(&self) -> &'static str {
        match self {
            Self::AssetSchema => "aws/asset_schema.yaml",
            Self::CreateAction => "aws/create_action.yaml",
            Self::DestroyAction => "aws/destroy_action.yaml",
            Self::RefreshAction => "aws/refresh_action.yaml",
            Self::UpdateAction => "aws/update_action.yaml",
        }
    }

    fn raw_prompt_yaml_embedded(&self) -> &'static str {
        match self {
            Self::AssetSchema => include_str!("../prompts/aws/asset_schema.yaml"),
            Self::CreateAction => include_str!("../prompts/aws/create_action.yaml"),
            Self::DestroyAction => include_str!("../prompts/aws/destroy_action.yaml"),
            Self::RefreshAction => include_str!("../prompts/aws/refresh_action.yaml"),
            Self::UpdateAction => include_str!("../prompts/aws/update_action.yaml"),
        }
    }
}

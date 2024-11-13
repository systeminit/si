use std::{borrow::Cow, path::PathBuf};

use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{AssetSprayerError, Result};

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
    strum::Display,
    strum::EnumString,
    strum::VariantNames,
)]
pub enum AwsCliCommandPromptKind {
    AssetSchema,
    CreateAction,
    DestroyAction,
    RefreshAction,
    UpdateAction,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AwsCliCommand(pub String, pub String);

impl Prompt {
    pub async fn prompt(
        &self,
        prompts_dir: &Option<PathBuf>,
    ) -> Result<CreateChatCompletionRequest> {
        let raw_prompt = self.raw_prompt(prompts_dir).await?;
        self.replace_prompt(raw_prompt).await
    }

    async fn replace_prompt(
        &self,
        request: CreateChatCompletionRequest,
    ) -> Result<CreateChatCompletionRequest> {
        let mut request = request;
        for message in request.messages.iter_mut() {
            *message = self.replace_prompt_message(message.clone()).await?;
        }
        Ok(request)
    }

    async fn replace_prompt_message(
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
            }) => *text = self.replace_prompt_text(text).await?,
            _ => {}
        };
        Ok(message)
    }

    async fn replace_prompt_text(&self, text: &str) -> Result<String> {
        let text = match self {
            Self::AwsCliCommandPrompt(_, command) => text
                .replace("{AWS_COMMAND}", command.command())
                .replace("{AWS_SUBCOMMAND}", command.subcommand()),
        };
        self.fetch_prompt_text(&text).await
    }

    async fn fetch_prompt_text(&self, text: &str) -> Result<String> {
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

    async fn get(url: &str) -> reqwest::Result<String> {
        info!("Fetching: {}", url);
        let client = reqwest::ClientBuilder::new()
            .user_agent("Wget/1.21.2")
            .build()?;
        let response = client.get(url).send().await?;
        response.error_for_status()?.text().await
    }

    pub async fn raw_prompt(
        &self,
        prompts_dir: &Option<PathBuf>,
    ) -> Result<CreateChatCompletionRequest> {
        Ok(serde_yaml::from_str(
            &self.raw_prompt_yaml(prompts_dir).await?,
        )?)
    }

    async fn raw_prompt_yaml(&self, prompts_dir: &Option<PathBuf>) -> Result<Cow<'static, str>> {
        if let Some(ref prompts_dir) = prompts_dir {
            // Read from disk if prompts_dir is available (faster dev cycle)
            let path = prompts_dir.join(self.raw_prompt_yaml_relative_path());
            info!("Loading prompt for {} from disk at {:?}", self, path);
            Ok(tokio::fs::read_to_string(path).await?.into())
        } else {
            info!("Loading embedded prompt for {}", self);
            Ok(self.raw_prompt_yaml_embedded().into())
        }
    }

    fn raw_prompt_yaml_relative_path(&self) -> &str {
        match self {
            Self::AwsCliCommandPrompt(kind, _) => kind.yaml_relative_path(),
        }
    }

    fn raw_prompt_yaml_embedded(&self) -> &'static str {
        match self {
            Self::AwsCliCommandPrompt(kind, _) => kind.yaml_embedded(),
        }
    }
}

impl AwsCliCommandPromptKind {
    const fn yaml_relative_path(&self) -> &'static str {
        match self {
            Self::AssetSchema => "aws/asset_schema.yaml",
            Self::CreateAction => "aws/create_action.yaml",
            Self::DestroyAction => "aws/destroy_action.yaml",
            Self::RefreshAction => "aws/refresh_action.yaml",
            Self::UpdateAction => "aws/update_action.yaml",
        }
    }

    fn yaml_embedded(&self) -> &'static str {
        match self {
            Self::AssetSchema => include_str!("../prompts/aws/asset_schema.yaml"),
            Self::CreateAction => include_str!("../prompts/aws/create_action.yaml"),
            Self::DestroyAction => include_str!("../prompts/aws/destroy_action.yaml"),
            Self::RefreshAction => include_str!("../prompts/aws/refresh_action.yaml"),
            Self::UpdateAction => include_str!("../prompts/aws/update_action.yaml"),
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

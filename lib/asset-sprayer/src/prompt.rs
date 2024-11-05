use std::{borrow::Cow, path::PathBuf};

use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
};
use telemetry::prelude::*;

use crate::{AssetSprayerError, Result};

#[derive(Debug, Clone, strum::Display, strum::EnumDiscriminants)]
#[strum_discriminants(name(PromptKind))]
#[strum_discriminants(derive(strum::Display))]
pub enum Prompt {
    AwsAssetSchema { command: String, subcommand: String },
}

impl Prompt {
    pub fn kind(&self) -> PromptKind {
        self.into()
    }

    pub async fn prompt(
        &self,
        prompts_dir: &Option<PathBuf>,
    ) -> Result<CreateChatCompletionRequest> {
        let raw_prompt = self.kind().raw_prompt(prompts_dir).await?;
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
            Self::AwsAssetSchema {
                command,
                subcommand,
            } => text
                .replace("{AWS_COMMAND}", command)
                .replace("{AWS_SUBCOMMAND}", subcommand),
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
                return Err(AssetSprayerError::MissingEndFetch(self.kind()));
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
}

impl PromptKind {
    pub async fn raw_prompt(
        &self,
        prompts_dir: &Option<PathBuf>,
    ) -> Result<CreateChatCompletionRequest> {
        Ok(serde_yaml::from_str(&self.yaml(prompts_dir).await?)?)
    }

    async fn yaml(&self, prompts_dir: &Option<PathBuf>) -> Result<Cow<'static, str>> {
        if let Some(ref prompts_dir) = prompts_dir {
            // Read from disk if prompts_dir is available (faster dev cycle)
            let path = prompts_dir.join(self.yaml_relative_path());
            info!("Loading prompt for {} from disk at {:?}", self, path);
            Ok(tokio::fs::read_to_string(path).await?.into())
        } else {
            info!("Loading embedded prompt for {}", self);
            Ok(self.yaml_embedded().into())
        }
    }

    fn yaml_relative_path(&self) -> &str {
        match self {
            Self::AwsAssetSchema => "aws/asset_schema.yaml",
        }
    }

    fn yaml_embedded(&self) -> &'static str {
        match self {
            Self::AwsAssetSchema => include_str!("../prompts/aws/asset_schema.yaml"),
        }
    }
}

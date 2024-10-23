use std::path::PathBuf;

use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageContent,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
};
use telemetry::prelude::*;

use crate::{AssetSprayerError, AssetSprayerResult};

#[derive(Debug, Clone)]
pub struct Prompts {
    prompts_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, strum::Display)]
pub enum Prompt {
    AssetSchema,
}

impl Prompt {
    fn yaml_relative_path(&self) -> &str {
        match self {
            Prompt::AssetSchema => "aws/asset_schema.yaml",
        }
    }

    fn yaml_embedded(&self) -> &'static str {
        match self {
            Prompt::AssetSchema => include_str!("../prompts/aws/asset_schema.yaml"),
        }
    }
}

impl Prompts {
    pub fn new(prompts_dir: Option<PathBuf>) -> Self {
        Self {
            prompts_dir: prompts_dir.map(Into::into),
        }
    }

    pub async fn create_request(
        &self,
        prompt: Prompt,
        replace: &[(&str, &str)],
    ) -> AssetSprayerResult<CreateChatCompletionRequest> {
        let request = self.raw_request(prompt).await?;
        Self::replace_prompt_request(request, replace, prompt).await
    }

    async fn raw_request(&self, prompt: Prompt) -> AssetSprayerResult<CreateChatCompletionRequest> {
        Ok(serde_yaml::from_str(&self.yaml(prompt).await?)?)
    }

    async fn yaml(&self, prompt: Prompt) -> AssetSprayerResult<String> {
        if let Some(ref prompts_dir) = self.prompts_dir {
            // Read from disk if prompts_dir is available (faster dev cycle)
            let path = prompts_dir.join(prompt.yaml_relative_path());
            info!("Loading prompt for {} from disk at {:?}", prompt, path);
            Ok(tokio::fs::read_to_string(path).await?)
        } else {
            info!("Loading embedded prompt for {}", prompt);
            Ok(prompt.yaml_embedded().to_string())
        }
    }

    async fn replace_prompt_request(
        request: CreateChatCompletionRequest,
        replace: &[(&str, &str)],
        prompt: Prompt,
    ) -> AssetSprayerResult<CreateChatCompletionRequest> {
        let mut request = request;
        for message in request.messages.iter_mut() {
            *message =
                Self::replace_prompt_request_message(message.clone(), replace, prompt).await?;
        }
        Ok(request)
    }

    async fn replace_prompt_request_message(
        message: ChatCompletionRequestMessage,
        replace: &[(&str, &str)],
        prompt: Prompt,
    ) -> AssetSprayerResult<ChatCompletionRequestMessage> {
        let mut message = message;
        match &mut message {
            ChatCompletionRequestMessage::User(message) => {
                if let ChatCompletionRequestUserMessageContent::Text(text) = &mut message.content {
                    *text = Self::replace_prompt_text(text.clone(), replace, prompt).await?;
                }
            }
            ChatCompletionRequestMessage::System(message) => {
                if let ChatCompletionRequestSystemMessageContent::Text(text) = &mut message.content
                {
                    *text = Self::replace_prompt_text(text.clone(), replace, prompt).await?;
                }
            }
            _ => (),
        }
        Ok(message)
    }

    async fn replace_prompt_text(
        text: String,
        replace: &[(&str, &str)],
        prompt: Prompt,
    ) -> AssetSprayerResult<String> {
        let mut text = text;

        // Replace {KEY} with value
        for (from, to) in replace {
            text = text.replace(from, to);
        }

        Self::fetch_prompt_text(&text, prompt).await
    }

    async fn fetch_prompt_text(text: &str, prompt: Prompt) -> AssetSprayerResult<String> {
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
                return Err(AssetSprayerError::MissingEndFetch(prompt));
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

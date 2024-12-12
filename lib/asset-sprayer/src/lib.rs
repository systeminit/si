//! This create provides centralized support for using AI to generate assets.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    // missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::{borrow::Cow, path::PathBuf};

use async_openai::config::OpenAIConfig;
use config::AssetSprayerConfig;
use prompt::AwsCliCommandPromptKind;
use telemetry::prelude::*;
use thiserror::Error;

pub mod config;
pub mod prompt;
pub use prompt::Prompt;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AssetSprayerError {
    #[error("Empty choice returned from AI.")]
    EmptyChoice,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Missing end {{/FETCH}} after {{FETCH}}: {0}")]
    MissingEndFetch(Prompt),
    #[error("No choices were returned from the AI.")]
    NoChoices,
    #[error("OpenAI error: {0}")]
    OpenAI(#[from] async_openai::error::OpenAIError),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("SerdeYaml error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("Unreachable")]
    Unreachable,
}

pub type Result<T> = std::result::Result<T, AssetSprayerError>;

#[derive(Debug, Clone)]
pub struct AssetSprayer {
    pub openai_client: async_openai::Client<OpenAIConfig>,
    pub prompts_dir: Option<PathBuf>,
}

impl AssetSprayer {
    pub fn new(
        openai_client: async_openai::Client<OpenAIConfig>,
        config: AssetSprayerConfig,
    ) -> Self {
        Self {
            openai_client,
            prompts_dir: config.prompts_dir.map(Into::into),
        }
    }

    pub async fn raw_prompt(&self, kind: AwsCliCommandPromptKind) -> Result<Cow<'static, str>> {
        if let Some(ref prompts_dir) = self.prompts_dir {
            // Read from disk if prompts_dir is available (faster dev cycle)
            let path = prompts_dir.join(kind.raw_prompt_yaml_relative_path());
            info!("Loading prompt for {} from disk at {:?}", kind, path);
            Ok(tokio::fs::read_to_string(path).await?.into())
        } else {
            info!("Loading embedded prompt for {}", kind);
            Ok(kind.raw_prompt_yaml_embedded().into())
        }
    }

    pub async fn run(
        &self,
        prompt: &Prompt,
        raw_prompt: &str,
        function_text: &str,
    ) -> Result<String> {
        debug!("Generating {}", prompt);
        let mut function_text = function_text.to_string();
        for raw_request in prompt.read(raw_prompt)? {
            let request = prompt.generate(raw_request, &function_text).await?;
            let response = self.openai_client.chat().create(request).await?;
            let choice = response
                .choices
                .into_iter()
                .next()
                .ok_or(AssetSprayerError::NoChoices)?;
            function_text = choice
                .message
                .content
                .ok_or(AssetSprayerError::EmptyChoice)?;
        }
        Ok(function_text)
    }
}

pub trait RawPromptYamlSource {
    fn raw_prompt_yaml_relative_path(&self) -> &'static str;
    fn raw_prompt_yaml_embedded(&self) -> &'static str;
}

#[ignore = "You must have OPENAI_API_KEY set to run this test"]
#[tokio::test]
async fn test_do_ai() -> Result<()> {
    let asset_sprayer =
        AssetSprayer::new(async_openai::Client::new(), AssetSprayerConfig::default());
    let prompt = Prompt::AwsCliCommandPrompt(
        prompt::AwsCliCommandPromptKind::AssetSchema,
        prompt::AwsCliCommand::new("sqs", "create-queue"),
    );
    let raw_prompt = asset_sprayer.raw_prompt(prompt.kind()).await?;
    println!(
        "Done: {}",
        asset_sprayer.run(&prompt, &raw_prompt, "").await?
    );
    Ok(())
}

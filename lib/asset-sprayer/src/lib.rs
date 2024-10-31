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

use async_openai::config::OpenAIConfig;
use config::AssetSprayerConfig;
use prompts::{Prompt, Prompts};
use telemetry::prelude::*;
use thiserror::Error;

pub mod config;
pub mod prompts;

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

pub type AssetSprayerResult<T> = Result<T, AssetSprayerError>;

#[derive(Debug, Clone)]
pub struct AssetSprayer {
    openai_client: async_openai::Client<OpenAIConfig>,
    prompts: Prompts,
}

impl AssetSprayer {
    pub fn new(
        openai_client: async_openai::Client<OpenAIConfig>,
        config: AssetSprayerConfig,
    ) -> Self {
        Self {
            openai_client,
            prompts: Prompts::new(config.prompts_dir),
        }
    }

    pub async fn aws_asset_schema(
        &self,
        aws_command: &str,
        aws_subcommand: &str,
    ) -> AssetSprayerResult<String> {
        debug!(
            "Generating asset schema for 'aws {} {}'",
            aws_command, aws_subcommand
        );
        self.run(
            Prompt::AssetSchema,
            &[
                ("{AWS_COMMAND}", aws_command),
                ("{AWS_SUBCOMMAND}", aws_subcommand),
            ],
        )
        .await
    }

    async fn run(&self, prompt: Prompt, replace: &[(&str, &str)]) -> AssetSprayerResult<String> {
        let request = self.prompts.create_request(prompt, replace).await?;
        let response = self.openai_client.chat().create(request).await?;
        let choice = response
            .choices
            .into_iter()
            .next()
            .ok_or(AssetSprayerError::NoChoices)?;
        let text = choice
            .message
            .content
            .ok_or(AssetSprayerError::EmptyChoice)?;
        Ok(text)
    }
}

#[ignore = "You must have OPENAI_API_KEY set to run this test"]
#[tokio::test]
async fn test_do_ai() -> AssetSprayerResult<()> {
    let asset_sprayer =
        AssetSprayer::new(async_openai::Client::new(), AssetSprayerConfig::default());
    println!(
        "Done: {}",
        asset_sprayer
            .aws_asset_schema("sqs", "create-queue")
            .await?
    );
    Ok(())
}

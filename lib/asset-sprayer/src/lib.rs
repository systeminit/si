//! This create provides centralized logic for working with the billing events NATS Jetstream stream.

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
    missing_docs,
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

use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
    },
    Client,
};
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AssetSprayerError {
    #[error("Empty choice returned from AI.")]
    EmptyChoice,
    #[error("No choices were returned from the AI.")]
    NoChoices,
    #[error("OpenAI error: {0}")]
    OpenAI(#[from] async_openai::error::OpenAIError),
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

pub type AssetSprayerResult<T> = Result<T, AssetSprayerError>;
const INSTRUCTIONS: &str = include_str!("instructions.txt");
// const SI_SCHEMA_DOC: &str = include_str!("../../../app/docs/src/reference/asset/schema.md");
const SI_SCHEMA_DOC_URL: &str = "https://raw.githubusercontent.com/systeminit/si/refs/heads/main/app/docs/src/reference/asset/schema.md";
const AWS_CLI_BASE_URL: &str = "https://docs.aws.amazon.com/cli/latest/reference/";

pub struct NoChoicesError;
impl std::fmt::Display for NoChoicesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No choices were returned from the AI.")
    }
}

pub async fn si_schema_doc() -> AssetSprayerResult<String> {
    Ok(format!("The official documentation for System Initiative asset schemas, retrieved from {}, is after the colon and takes up the rest of this message: {}", SI_SCHEMA_DOC_URL, reqwest::get(SI_SCHEMA_DOC_URL).await?.text().await?))
}

pub async fn aws_cli_doc_message(command: &str, subcommand: &str) -> AssetSprayerResult<String> {
    let url = format!(
        "https://docs.aws.amazon.com/cli/latest/reference/{}/{}.html",
        command, subcommand
    );
    let doc = reqwest::get(&url).await?.text().await?;
    Ok(format!("The official HTML documentation for 'aws {} {}', retrieved from {}, is after the colon and takes up the rest of this message: {}", command, subcommand, url, doc))
}

fn system_message(
    content: impl Into<ChatCompletionRequestSystemMessageContent>,
) -> AssetSprayerResult<ChatCompletionRequestMessage> {
    Ok(ChatCompletionRequestSystemMessageArgs::default()
        .content(content)
        .build()?
        .into())
}

fn user_message(
    content: impl Into<ChatCompletionRequestUserMessageContent>,
) -> AssetSprayerResult<ChatCompletionRequestMessage> {
    Ok(ChatCompletionRequestUserMessageArgs::default()
        .content(content)
        .build()?
        .into())
}

pub async fn do_ai() -> AssetSprayerResult<String> {
    let client = Client::new();

    // Create request using builder pattern
    // Every request struct has companion builder struct with same name + Args suffix
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        // .max_prompt_tokens(...)
        // .max_completion_tokens(...)
        .messages([
            system_message(INSTRUCTIONS)?,
            user_message(aws_cli_doc_message("sqs", "create-queue").await?)?,
            user_message("Create the asset function that generates the schema.")?,
        ])
        .temperature(0f32)
        .build()?;

    // Call API
    let response = client.chat().create(request).await?;
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

#[tokio::test]
async fn test_do_ai() -> AssetSprayerResult<()> {
    println!("Done: {}", do_ai().await?);
    Ok(())
}

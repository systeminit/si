use std::path::PathBuf;

use asset_sprayer::{
    config::AssetSprayerConfig,
    prompt::{AwsCliCommand, AwsCliCommandPromptKind, Prompt},
    AssetSprayer,
};
use clap::{Parser, ValueEnum};
use color_eyre::Result;

const NAME: &str = "asset-sprayer-prompts";

#[derive(Parser, Debug)]
#[command(name = NAME, max_term_width = 100)]
pub(crate) struct Args {
    /// The action to take with the prompt.
    #[arg(long, short = 'a', value_enum, default_value = "run")]
    pub action: Action,

    /// The kind of prompt.
    #[arg(index = 1, value_enum)]
    pub prompt_kind: AwsCliCommandPromptKind,
    /// The AWS command to generate a function for.
    #[arg(index = 2)]
    pub aws_command: String,
    /// The AWS subcommand to generate a function for.
    #[arg(index = 3)]
    pub aws_subcommand: String,

    /// Directory to load prompts from.
    #[arg(long, env = "SI_ASSET_SPRAYER_PROMPTS_DIR")]
    pub prompts_dir: Option<String>,
}

/// The action to take with the prompt.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Action {
    /// Show the prompt.
    Show,
    /// Run the prompt.
    Run,
}

fn parse_args() -> Result<Args> {
    let mut args = Args::parse();
    if args.prompts_dir.is_none() {
        // If we're in a localdev environment, use local prompts_dir
        #[allow(clippy::disallowed_methods)] // for std::env::var(), this is a binary
        if let Ok(direnv_dir) = std::env::var("DIRENV_DIR") {
            // Check if DIRENV_DIR/lib/asset-sprayer/prompts exists
            let prompts_dir = PathBuf::from(direnv_dir).join("lib/asset-sprayer/prompts");
            if prompts_dir.try_exists()? {
                args.prompts_dir = Some(prompts_dir.to_string_lossy().into());
            }
        }
    }
    Ok(args)
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = parse_args()?;

    let asset_sprayer = AssetSprayer::new(
        async_openai::Client::new(),
        AssetSprayerConfig {
            prompts_dir: args.prompts_dir,
        },
    );
    let aws_command = AwsCliCommand::new(args.aws_command, args.aws_subcommand);
    let prompt = Prompt::AwsCliCommandPrompt(args.prompt_kind, aws_command);
    let raw_prompt = asset_sprayer.raw_prompt(args.prompt_kind).await?;
    match args.action {
        Action::Show => {
            let prompt = prompt.generate(&raw_prompt).await?;
            println!("{}", serde_yaml::to_string(&prompt)?);
        }
        Action::Run => {
            let asset_schema = asset_sprayer.run(&prompt, &raw_prompt).await?;
            println!("{}", asset_schema);
        }
    }
    Ok(())
}

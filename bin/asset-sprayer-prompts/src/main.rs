use asset_sprayer::{config::AssetSprayerConfig, prompt::Prompt, AssetSprayer};
use clap::{Parser, ValueEnum};
use color_eyre::Result;

const NAME: &str = "asset-sprayer-prompts";

#[derive(Parser, Debug)]
#[command(name = NAME, max_term_width = 100)]
pub(crate) struct Args {
    /// The action to take with the prompt.
    #[arg(index = 1, value_enum)]
    pub action: Action,
    /// The AWS command to generate an asset schema for.
    #[arg(index = 2)]
    pub aws_command: String,
    /// The AWS subcommand to generate an asset schema for.
    #[arg(index = 3)]
    pub aws_subcommand: String,
    /// Directory to load prompts from.
    #[arg(long)]
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

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let asset_sprayer = AssetSprayer::new(
        async_openai::Client::new(),
        AssetSprayerConfig {
            prompts_dir: args.prompts_dir,
        },
    );
    let prompt = Prompt::AwsAssetSchema {
        command: args.aws_command.clone(),
        subcommand: args.aws_subcommand.clone(),
    };
    match args.action {
        Action::Show => {
            let prompt = asset_sprayer.prompt(&prompt).await?;
            println!("{}", serde_yaml::to_string(&prompt)?);
        }
        Action::Run => {
            let asset_schema = asset_sprayer.run(&prompt).await?;
            println!("{}", asset_schema);
        }
    }
    Ok(())
}

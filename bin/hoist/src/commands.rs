use std::path::PathBuf;

use clap::Subcommand;

#[derive(Subcommand, Debug)]
#[remain::sorted]
pub enum Commands {
    AnonymizeSpecs(AnonymizeSpecsArgs),
    CompareSpecs(CompareSpecsArgs),
    GetDiffForAsset(GetDiffForAssetArgs),
    GetDiffSummary(GetDiffSummaryArgs),
    UploadAllSpecs(UploadAllSpecsArgs),
    UploadSpec(UploadSpecArgs),
    ValidateSpecs(ValidateSpecsArgs),
    WriteAllSpecs(WriteAllSpecsArgs),
    WriteExistingModulesSpec(WriteExistingModulesSpecArgs),
    WriteSpec(WriteSpecArgs),
}

#[derive(clap::Args, Debug)]
#[command(about = "Upload all specs in {target_dir} to the module index")]
pub struct CompareSpecsArgs {
    #[arg(
        long,
        short = 's',
        required = true,
        help = "Path to the first spec to compare"
    )]
    pub source_path: PathBuf,

    #[arg(
        long,
        short = 't',
        required = true,
        help = "Path to the second spec to compare"
    )]
    pub target_path: PathBuf,
}
#[derive(clap::Args, Debug)]
#[command(about = "Generate an anonymized version of target spec(s)")]
pub struct AnonymizeSpecsArgs {
    #[arg(long, short = 'o', required = true)]
    pub out: PathBuf,

    #[arg(
        long,
        short = 't',
        required = true,
        help = "Path to the directory containing specs to anonymize"
    )]
    pub target_dir: PathBuf,

    #[arg(
        long,
        default_value = "100",
        help = "Maximum number of concurrent uploads."
    )]
    pub max_concurrent: usize,
}

#[derive(clap::Args, Debug)]
#[command(about = "Compare specs in {target_dir} to the module index and generate summary")]
pub struct GetDiffSummaryArgs {
    #[arg(
        long,
        short = 't',
        required = true,
        help = "Path to the directory containing specs to diff"
    )]
    pub target_dir: PathBuf,
}

#[derive(clap::Args, Debug)]
#[command(
    about = "Compare single spec in {target_dir} to the module index and generate detailed changelog"
)]
pub struct GetDiffForAssetArgs {
    #[arg(long, short = 't', required = true, help = "Path to the spec to diff")]
    pub target_path: PathBuf,
}

#[derive(clap::Args, Debug)]
#[command(about = "Upload all specs in {target_dir} to the module index")]
pub struct UploadAllSpecsArgs {
    #[arg(
        long,
        short = 't',
        required = true,
        help = "Path to the directory containing specs to upload"
    )]
    pub target_dir: PathBuf,

    #[arg(
        long,
        default_value = "100",
        help = "Maximum number of concurrent uploads."
    )]
    pub max_concurrent: usize,

    #[arg(
        long = "skip-confirmation",
        short = 'y',
        help = "Skip confirmation prompts",
        action = clap::ArgAction::SetTrue
    )]
    pub skip_confirmation: bool,

    #[arg(
        long = "non-interactive",
        short = 'v',
        help = "Write to console instead of progress bar",
        action = clap::ArgAction::SetTrue
    )]
    pub non_interactive: bool,
}

#[derive(clap::Args, Debug)]
#[command(about = "Upload the spec {target} to the module index")]
pub struct UploadSpecArgs {
    #[arg(
        long,
        short = 't',
        required = true,
        help = "Path to the spec to upload"
    )]
    pub target: PathBuf,

    #[arg(
        long,
        default_value = "100",
        help = "Maximum number of concurrent uploads."
    )]
    pub max_concurrent: usize,

    #[arg(
        long = "skip-confirmation",
        short = 'y',
        help = "Skip confirmation prompts",
        action = clap::ArgAction::SetTrue
    )]
    pub skip_confirmation: bool,

    #[arg(
        long = "non-interactive",
        short = 'v',
        help = "Write to console instead of progress bar",
        action = clap::ArgAction::SetTrue
    )]
    pub non_interactive: bool,
}

#[derive(clap::Args, Debug)]
#[command(about = "Validate that specs in {target_dir} can be loaded and converted")]
pub struct ValidateSpecsArgs {
    #[arg(
        long,
        short = 't',
        required = true,
        help = "Path to the directory containing specs to validate"
    )]
    pub target_dir: PathBuf,
}

#[derive(clap::Args, Debug)]
#[command(about = "Get all built-ins an write out a hashmap with their name and schema id")]
pub struct WriteExistingModulesSpecArgs {
    #[arg(long, short = 'o', required = true)]
    pub out: PathBuf,
}

#[derive(clap::Args, Debug)]
#[command(about = "Get {spec_name} from the module index and write it to {out}")]
pub struct WriteSpecArgs {
    #[arg(long, short = 's', required = true)]
    pub spec_name: String,
    #[arg(long, short = 'o', required = true)]
    pub out: PathBuf,
}

#[derive(clap::Args, Debug)]
#[command(about = "Get all specs from the module index and write them to {out}")]
pub struct WriteAllSpecsArgs {
    #[arg(long, short = 'o', required = true)]
    pub out: PathBuf,
}

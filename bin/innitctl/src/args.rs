use std::path::PathBuf;

use clap::{
    ArgAction,
    Parser,
};
use innitctl_backend::{
    Config,
    ConfigError,
    ConfigFile,
    ConfigMap,
    ParameterProvider,
    StandardConfigFile,
};
use si_service::prelude::*;

pub const NAME: &str = "innitctl";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// Arguments for the innitctl service.
#[derive(Parser, Debug)]
#[command(name = NAME, max_term_width = 100)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 6.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Disables ANSI coloring in log output, even if standard output refers to a terminal/TTY.
    ///
    /// For more details, visit: <http://no-color.org/>.
    #[arg(
        long = "no-color",
        default_value = "false",
        env = "SI_NO_COLOR",
        hide_env_values = true,
        conflicts_with = "force_color"
    )]
    pub(crate) no_color: bool,

    /// Forces ANSI coloring, even if standard output refers to a terminal/TTY.
    ///
    /// For more details, visit: <http://no-color.org/>.
    #[arg(
        long = "force-color",
        default_value = "false",
        env = "SI_FORCE_COLOR",
        hide_env_values = true,
        conflicts_with = "no_color"
    )]
    pub(crate) force_color: bool,

    /// Prints telemetry logging as JSON lines.
    ///
    /// For more details, visit: <https://jsonlines.org/>.
    #[arg(
        long = "log-json",
        default_value = "false",
        env = "SI_LOG_JSON",
        hide_env_values = true
    )]
    pub(crate) log_json: bool,

    /// Additionally appends logging to rolling files under the given directory.
    #[arg(
        long = "log-file-directory",
        env = "SI_LOG_FILE_DIRECTORY",
        hide_env_values = true
    )]
    pub(crate) log_file_directory: Option<PathBuf>,

    /// Directory containing the config files to templatize
    #[arg(long)]
    pub(crate) configs: Option<PathBuf>,

    /// Directory to output rendered configs to
    #[arg(long)]
    pub(crate) output: Option<PathBuf>,

    /// Host environment template value
    #[arg(long)]
    pub(crate) host_environment: Option<String>,

    /// Instance id template value
    #[arg(long)]
    pub(crate) instance_id: Option<String>,

    /// Service name template value
    #[arg(long)]
    pub(crate) service_name: Option<String>,
}

fn build_config_map(args: Args, config_map: &mut ConfigMap) -> &ConfigMap {
    if let Some(configs) = args.configs {
        config_map.set("config_directory", configs.display().to_string());
    }
    if let Some(output) = args.output {
        config_map.set("output_directory", output.display().to_string());
    }
    if let Some(hostenv) = args.host_environment {
        config_map.set("host_environment", hostenv);
    }
    if let Some(instance_id) = args.instance_id {
        config_map.set("instance_id", instance_id);
    }
    if let Some(service_name) = args.service_name {
        config_map.set("service_name", service_name);
    }
    config_map
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, |config_map| {
            build_config_map(args, config_map);
        })?
        .try_into()
    }
}

pub async fn load_config_with_provider<P>(
    args: Args,
    provider: Option<P>,
) -> Result<Config, ConfigError>
where
    P: ParameterProvider + 'static,
{
    ConfigFile::layered_load_with_provider::<_, P>(NAME, provider, move |config_map| {
        build_config_map(args, config_map);
    })
    .await?
    .try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_command() {
        use clap::CommandFactory;
        Args::command().debug_assert()
    }
}

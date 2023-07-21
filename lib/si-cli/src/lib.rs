use color_eyre::Result;
use console::Emoji;
use thiserror::Error;

pub mod cmd;

static PACKAGES: &[&str] = &[
    "systeminit/sdf",
    "systeminit/council",
    "systeminit/veritech",
    "systeminit/pinga",
    "systeminit/web",
    "jaeger",
    "otelcol",
    "postgres",
    "nats",
];

static START_COMMANDS: &[&str] = &["docker run"];
static STOP_COMMANDS: &[&str] = &["docker stop"];
static RESTART_COMMANDS: &[&str] = &["docker restart"];

static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SiCliError {
    #[error("failed to launch web url {0}")]
    FailToLaunch(String),
    #[error("incorrect installation type {0}")]
    IncorrectInstallMode(String),
}

pub type CliResult<T> = Result<T, SiCliError>;

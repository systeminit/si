use std::path::PathBuf;

use clap::Parser;
use si_data_nats::Subject;

pub const NAME: &str = "nats-stream-copy-data";

include!(concat!(env!("OUT_DIR"), "/git_metadata.rs"));

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// Copy messages from one NATS stream to another
#[derive(Parser, Debug)]
#[command(name = NAME, version = VERSION, max_term_width = 100)]
pub(crate) struct Args {
    /// Source Stream to copy messages from
    pub(crate) source_stream: String,

    /// Destination Stream for copied messages
    pub(crate) destination_stream: String,

    /// Subject filter from source Stream
    #[arg(short, long)]
    pub(crate) subject: Vec<Subject>,

    /// NATS connection URL for source and destination Streams [example: demo.nats.io]
    #[arg(
        long,
        default_value = Self::DEFAULT_NATS_URL,
        env = "NATS_URL",
        hide_env_values = true,
        conflicts_with_all = [
            "source_nats_url",
            "destination_nats_url",
        ]
    )]
    pub(crate) nats_url: Option<String>,

    /// NATS credentials file for source and destination Streams
    #[arg(long, env = "NATS_CREDS", hide_env_values = true)]
    pub(crate) nats_creds: Option<PathBuf>,

    /// NATS connection URL for source Stream [example: demo.nats.io]
    #[arg(long, requires = "destination_nats_url", conflicts_with = "nats_url")]
    pub(crate) source_nats_url: Option<String>,

    /// NATS credentials file for source Stream
    #[arg(long)]
    pub(crate) source_nats_creds: Option<PathBuf>,

    /// NATS connection URL for destination Streams [example: demo.nats.io]
    #[arg(long, requires = "source_nats_url", conflicts_with = "nats_url")]
    pub(crate) destination_nats_url: Option<String>,

    /// NATS credentials file for destination Stream
    #[arg(long)]
    pub(crate) destination_nats_creds: Option<PathBuf>,
}

impl Args {
    pub(crate) const DEFAULT_NATS_URL: &str = "nats://localhost:4222";

    pub(crate) fn source_nats_url(&self) -> String {
        match self.source_nats_url.as_ref() {
            Some(url) => url.to_string(),
            None => self
                .nats_url
                .as_ref()
                .map(|url| url.to_string())
                .unwrap_or_else(|| Self::DEFAULT_NATS_URL.to_string()),
        }
    }

    pub(crate) fn source_nats_creds(&self) -> Option<String> {
        match self.source_nats_creds.as_deref() {
            Some(path) => Some(path),
            None => self.nats_creds.as_deref(),
        }
        .map(|path| path.to_string_lossy().to_string())
    }

    pub(crate) fn destination_nats_url(&self) -> String {
        match self.destination_nats_url.as_ref() {
            Some(url) => url.to_string(),
            None => self
                .nats_url
                .as_ref()
                .map(|url| url.to_string())
                .unwrap_or_else(|| Self::DEFAULT_NATS_URL.to_string()),
        }
    }

    pub(crate) fn destination_nats_creds(&self) -> Option<String> {
        match self.destination_nats_creds.as_deref() {
            Some(path) => Some(path),
            None => self.nats_creds.as_deref(),
        }
        .map(|path| path.to_string_lossy().to_string())
    }
}

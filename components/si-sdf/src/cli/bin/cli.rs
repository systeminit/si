use clap::{AppSettings, ArgSettings, Clap};
use lazy_static::lazy_static;
use regex::Regex;
use si_sdf::cli::server::command::change_run::NodeSetCommand;
use std::{fmt, str::FromStr};
use strum::VariantNames;
use strum_macros::{Display, EnumString, EnumVariantNames};
use thiserror::Error;
use url::Url;

const NAME: &str = env!("CARGO_BIN_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "The System Initiative <info@systeminit.com>\n\n";

pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The command line client for the System Initiative
///
/// https://www.systeminit.com/
#[derive(Clap, Debug)]
#[clap(
    global_setting(AppSettings::ColoredHelp),
    global_setting(AppSettings::UnifiedHelpMessage),
    max_term_width = 79,
    name = NAME,
    author = AUTHOR,
    version = VERSION
)]
pub(crate) struct Args {
    #[clap(subcommand)]
    pub(crate) subcmd: SubCmd,
}

#[derive(Clap, Debug)]
pub(crate) struct GlobalArgs {
    /// SI Client API host
    #[clap(
        short = 'H',
        long,
        default_value = "ws://localhost:5156",
        env = "SI_HOST",
        setting(ArgSettings::HideEnvValues)
    )]
    pub(crate) host: Url,

    /// SI Client token
    #[clap(
        short = 'T',
        long,
        env = "SI_TOKEN",
        setting(ArgSettings::HideEnvValues)
    )]
    pub(crate) token: String,
}

#[derive(Clap, Debug)]
pub(crate) enum SubCmd {
    Node(SubCmdNode),
}

/// Manage nodes
#[derive(Clap, Debug)]
pub(crate) enum SubCmdNode {
    Run(ArgsNodeRun),
}

/// Runs an action on a node in a system
///
/// Optionally, a set of properties can be set on specific nodes which allows you to update
/// versions, tags, etc. for a deployment for example.
#[derive(Clap, Debug)]
pub(crate) struct ArgsNodeRun {
    #[clap(flatten)]
    pub(crate) global: GlobalArgs,

    /// Output formatter to use
    #[clap(
        short = 'F',
        long,
        possible_values = OutputFormatter::VARIANTS,
        default_value = "simple",
        env = "SI_OUTPUT_FORMATTER",
        setting(ArgSettings::HideEnvValues)
    )]
    pub(crate) formatter: OutputFormatter,

    /// Set a property on a node [ex: node:bd345f3ec2dd41d3867d02bd9d5e2d8d.image=nginx:1.19.5]
    #[clap(short = 's', long)]
    pub(crate) set: Vec<NodeSet>,

    /// System id for the node
    #[clap(rename_all = "screaming_snake_case")]
    pub(crate) system_id: SystemId,

    /// Target node id
    #[clap(rename_all = "screaming_snake_case")]
    pub(crate) node_id: NodeId,

    /// Action to invoke [ex: deploy]
    #[clap(rename_all = "screaming_snake_case")]
    pub(crate) action: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SystemId(String);

impl fmt::Display for SystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Error)]
#[error("could not parse a system id")]
pub struct SystemIdParseError;

impl FromStr for SystemId {
    type Err = SystemIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new("^system:[a-f0-9]{32}$").unwrap();
        }

        if RE.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(SystemIdParseError)
        }
    }
}

impl From<SystemId> for String {
    fn from(value: SystemId) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(String);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Error)]
#[error("could not parse an entity id")]
pub struct NodeIdParseError;

impl FromStr for NodeId {
    type Err = NodeIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new("^node:[a-f0-9]{32}$").unwrap();
        }

        if RE.is_match(s) {
            Ok(Self(s.to_string()))
        } else {
            Err(NodeIdParseError)
        }
    }
}

impl From<NodeId> for String {
    fn from(value: NodeId) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NodeSet(NodeSetCommand);

#[derive(Debug, Error)]
pub enum NodeSetParseError {
    #[error("could not parse an node set command")]
    Malformed,
    #[error("could not parse node id in set command: {0}")]
    NodeId(NodeIdParseError),
}

impl FromStr for NodeSet {
    type Err = NodeSetParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut kv_parts = s.splitn(2, "=");
        let (key, raw_value) = match (kv_parts.next(), kv_parts.next(), kv_parts.next()) {
            (Some(key), Some(value), None) => (key, value),
            (_, _, _) => return Err(NodeSetParseError::Malformed),
        };
        let mut key_parts = key.splitn(2, ".");
        let (id, path_expr) = match (key_parts.next(), key_parts.next(), key_parts.next()) {
            (Some(node_id), Some(path), None) => (node_id, path),
            (_, _, _) => return Err(NodeSetParseError::Malformed),
        };

        let node_id = NodeId::from_str(id).map_err(NodeSetParseError::NodeId)?;
        let path: Vec<_> = path_expr.split(".").map(|s| s.to_string()).collect();
        let value = serde_json::json!(raw_value);

        Ok(Self(NodeSetCommand::new(node_id, path, value)))
    }
}

impl From<NodeSet> for NodeSetCommand {
    fn from(value: NodeSet) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Display, EnumString, EnumVariantNames, Eq, PartialEq)]
#[strum(serialize_all = "kebab_case")]
pub(crate) enum OutputFormatter {
    Debug,
    Simple,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod system_id {
        use super::*;
        #[test]
        fn from_str() {
            let id = SystemId::from_str("system:123e4567e89b12d3a456426614174000")
                .expect("should be valid");

            assert_eq!(
                SystemId("system:123e4567e89b12d3a456426614174000".to_string()),
                id
            )
        }

        #[test]
        fn from_str_invalid() {
            match SystemId::from_str("nope:123e4567e89b12d3a456426614174000") {
                Err(SystemIdParseError) => assert!(true),
                Ok(_) => panic!("should not succeed"),
            }
        }
    }

    mod node_id {
        use super::*;
        #[test]
        fn from_str() {
            let id =
                NodeId::from_str("node:123e4567e89b12d3a456426614174000").expect("should be valid");

            assert_eq!(
                NodeId("node:123e4567e89b12d3a456426614174000".to_string()),
                id
            )
        }

        #[test]
        fn from_str_invalid() {
            match NodeId::from_str("nope:123e4567e89b12d3a456426614174000") {
                Err(NodeIdParseError) => assert!(true),
                Ok(_) => panic!("should not succeed"),
            }
        }
    }
}

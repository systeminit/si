use std::{
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;
use color_eyre::{
    Result,
    eyre::{
        Error,
        eyre,
    },
};
use edda_client::Client;
use si_data_nats::{
    NatsClient,
    NatsConfig,
};
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use si_std::SensitiveString;

const NAME: &str = "rebuildinator";

#[derive(Parser, Debug)]
#[command(
    name = NAME,
    about = "Rebuilds MVs for a given list of workspace and change set pairs"
)]
struct Args {
    /// Pairs of workspace and change set identifiers in the following format: "<workspace-id>:<change-set-id>"
    workspace_and_change_set_pairs: Vec<WorkspaceAndChangeSetPair>,

    /// NATS connection URL [example: demo.nats.io]
    #[arg(long)]
    nats_url: Option<String>,

    /// NATS credentials string
    #[arg(long, allow_hyphen_values = true)]
    nats_creds: Option<SensitiveString>,

    /// NATS credentials file
    #[arg(long)]
    nats_creds_path: Option<PathBuf>,

    /// NATS subject prefix (typically used in tests)
    #[arg(long)]
    nats_subject_prefix: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse the args first because this can fail.
    let args = Args::parse();
    if args.workspace_and_change_set_pairs.is_empty() {
        return Err(eyre!("no workspace and change set pairs provided"));
    }

    // Create a NATS client.
    let mut nats_config = NatsConfig {
        connection_name: Some(NAME.to_string()),
        creds: args.nats_creds.map(|c| c.to_string()),
        creds_file: args.nats_creds_path.map(|p| p.display().to_string()),
        subject_prefix: args.nats_subject_prefix,
        ..Default::default()
    };
    if let Some(url) = args.nats_url {
        nats_config.url = url;
    }
    let nats = NatsClient::new(&nats_config).await?;

    // Create an edda client.
    let edda_client = Client::new(nats).await?;

    // Perform the requests!
    for WorkspaceAndChangeSetPair {
        workspace_id,
        change_set_id,
    } in args.workspace_and_change_set_pairs
    {
        println!("requesting for workspace '{workspace_id}' and change set '{change_set_id}'...");
        let request_id = edda_client
            .rebuild_for_change_set(workspace_id, change_set_id)
            .await?;
        println!("successfully sent the request with ID '{request_id}'");
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct WorkspaceAndChangeSetPair {
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
}

impl FromStr for WorkspaceAndChangeSetPair {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(eyre!(
                "invalid format (expected '<workspace-id>:<change-set-id>', got '{s}')"
            ));
        }

        let workspace_id: WorkspacePk = parts[0]
            .parse()
            .map_err(|e| eyre!("invalid workspace ID: '{}' (error: {e})", parts[0]))?;
        let change_set_id: ChangeSetId = parts[1]
            .parse()
            .map_err(|e| eyre!("invalid change set ID: '{}' (error: {e})", parts[1]))?;

        Ok(WorkspaceAndChangeSetPair {
            workspace_id,
            change_set_id,
        })
    }
}

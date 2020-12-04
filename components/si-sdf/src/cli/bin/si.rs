use anyhow::Result;
use cli::{ArgsNodeRun, OutputFormatter, SubCmd, SubCmdNode};
use si_sdf::cli::client::{Client, ClientCommand, DebugFormatter, NodeChangeRun, SimpleFormatter};

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::parse();

    match args.subcmd {
        SubCmd::Node(subcmd) => match subcmd {
            SubCmdNode::Run(args) => sub_node_run(args).await,
        },
    }
}

async fn sub_node_run(args: ArgsNodeRun) -> Result<()> {
    let mut url = args.global.host;
    if url.path() == "/" {
        url.set_path("cli");
    } else {
        url.set_path(&[url.path().replacen("/", "", 1), "cli".to_string()].join("/"));
    }
    url.set_query(Some(&format!("token=Bearer+{}", args.global.token)));

    let mut client = match args.formatter {
        OutputFormatter::Debug => Client::new(url, Box::new(DebugFormatter::new()))?,
        OutputFormatter::Simple => Client::new(url, Box::new(SimpleFormatter::new()))?,
    };

    let mut node_change_run = NodeChangeRun::new(args.node_id, args.system_id, args.action);
    for set_command in args.set.into_iter() {
        node_change_run.add_set_command(set_command.into());
    }

    let command = ClientCommand::NodeChangeRun(node_change_run);
    client.command(command).await?;

    Ok(())
}

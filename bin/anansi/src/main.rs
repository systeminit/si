use std::{
    fs::File,
    io::Read as _,
    path::PathBuf,
};

use app::{
    App,
    splash,
};
use clap::Parser;
use color_eyre::Result;
use dal::WorkspaceSnapshotGraph;
use si_layer_cache::db::serialize;
use tokio_util::sync::CancellationToken;

mod app;

#[derive(Parser, Debug)]
#[command(name = "anansi", version = "0.1.0")]
#[command(about = "anansi k(n)ows about graphs in system initiative")]
struct Args {
    /// Path to the snapshot you want to debug
    #[arg(required = true)]
    snapshot_path: PathBuf,
    /// Show the splash screen for longer (because pretty)
    #[arg(long, default_value_t = false)]
    long_splash: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    run_app(args).await?;

    Ok(())
}

async fn run_app(args: Args) -> Result<()> {
    let cancel_token = CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();
    let long_wait = args.long_splash;

    // Show splash while app inits
    let splash_future = tokio::spawn(async move {
        splash::show_splash(cancel_token, long_wait).await?;
        Ok::<(), color_eyre::Report>(())
    });

    let snapshot_path = args.snapshot_path.clone();
    let init_app_future = tokio::spawn(async move {
        let mut snap_file = File::open(&snapshot_path)?;
        let mut snap_bytes = vec![];
        snap_file.read_to_end(&mut snap_bytes)?;

        let graph: WorkspaceSnapshotGraph = serialize::from_bytes(&snap_bytes)?;
        let app = App::new(graph)?;

        // App has loaded, kill the splash screen
        cancel_token_clone.cancel();

        Ok::<_, color_eyre::Report>(app)
    });

    let mut app = init_app_future.await??;
    splash_future.await??;

    app.run().await?;

    Ok(())
}

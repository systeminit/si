use color_eyre::Result;
use tracing::debug;

mod args;
mod telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::init()?;
    let args = args::parse();
    debug!(arguments = ?args);

    println!("Hello, world!");

    Ok(())
}

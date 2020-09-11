use anyhow::Context;
use std::env;

use si_cea::binary::server::prelude::*;
use si_sdf::start;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "entities=info");
    }

    let server_name = "entity";
    println!("** Starting {} ***", server_name);
    setup_tracing("si-entity").context("failed to setup tracing")?;

    println!("*** Loading settings ***");
    let settings = si_settings::Settings::new()?;

    println!("*** Connecting to the database ***");
    let db = si_data::Db::new(&settings).context("failed to connect to the database")?;

    println!("*** Starting service ***");
    start(db, settings).await;

    Ok(())
}

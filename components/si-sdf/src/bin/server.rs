use std::env;

use si_data::{EventLogFS, NatsConn, PgPool};
use si_model::{jwt_key, migrate, Veritech};
use si_sdf::start;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "entities=info");
    }

    let server_name = "si-sdf";
    println!("*** Starting {} ***", server_name);
    println!("*** Loading settings ***");
    let settings = si_settings::Settings::new()?;

    println!("*** Connecting to postgres ***");
    let pg = PgPool::new(&settings.pg).await?;

    println!("*** Running migrations ***");
    migrate(&pg).await?;

    println!("*** Connecting to NATS ***");
    let nats = NatsConn::new(&settings.nats).await?;

    println!("*** Initializing EventLogFs ***");
    let event_log_fs = EventLogFS::init(&settings.event_log_fs).await?;

    println!("*** Initializing Veritech ***");
    let veritech = Veritech::new(&settings.veritech, event_log_fs.clone());

    println!("*** Checking for JWT keys ***");
    let mut conn = pg.pool.get().await?;
    let txn = conn.transaction().await?;
    jwt_key::create_jwt_key_if_missing(
        &txn,
        "config/public.pem",
        "config/private.pem",
        &settings.jwt_encrypt.key,
    )
    .await?;
    txn.commit().await?;

    println!("*** Starting service ***");
    start(pg, nats, veritech, event_log_fs, settings).await;

    Ok(())
}

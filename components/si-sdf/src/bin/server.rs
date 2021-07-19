use si_data::{EventLogFS, NatsConn, PgPool};
use si_model::{jwt_key, migrate, Veritech, Workflow};
use si_sdf::{start, telemetry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init()?;

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
    let veritech = Veritech::new(&settings.veritech, event_log_fs.clone()).await?;

    println!("*** Loading workflow builtins ***");
    Workflow::load_builtins(&pg, &veritech).await?;

    println!("*** Checking for JWT keys ***");
    let mut conn = pg.get().await?;
    let txn = conn.transaction().await?;
    jwt_key::create_jwt_key_if_missing(
        &txn,
        "config/public.pem",
        "config/private.pem",
        &settings.jwt_encrypt.key,
    )
    .await?;
    txn.commit().await?;

    println!("*** Starting resource scheduler ***");
    tokio::task::spawn(si_sdf::resource_scheduler::start(
        pg.clone(),
        nats.clone(),
        veritech.clone(),
    ));

    println!("*** Starting service ***");
    start(pg, nats, veritech, event_log_fs, settings).await;

    Ok(())
}

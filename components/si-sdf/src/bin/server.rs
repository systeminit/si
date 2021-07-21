use si_data::{EventLogFS, NatsConn, PgPool};
use si_model::{jwt_key, migrate, Veritech, Workflow};
use si_sdf::{start, telemetry};
use si_settings::Settings;
use tracing::{instrument, log::info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init()?;
    Server::init().await?.run().await
}

struct Server {
    pg: PgPool,
    nats: NatsConn,
    veritech: Veritech,
    event_log_fs: EventLogFS,
    settings: Settings,
}

impl Server {
    #[instrument(name = "server.init")]
    async fn init() -> anyhow::Result<Self> {
        let server_name = "si-sdf";
        info!("Starting {}", server_name);

        info!("Loading settings");
        let settings = si_settings::Settings::new()?;

        info!("Initializing connection pool to PostgreSQL");
        let pg = PgPool::new(&settings.pg).await?;

        info!("Running migrations");
        migrate(&pg).await?;

        info!("Initializing connection to NATS");
        let nats = NatsConn::new(&settings.nats).await?;

        info!("Initializing EventLogFs");
        let event_log_fs = EventLogFS::init(&settings.event_log_fs).await?;

        info!("Initializing Veritech client");
        let veritech = Veritech::new(&settings.veritech, event_log_fs.clone()).await?;

        info!("Loading workflow builtins");
        Workflow::load_builtins(&pg, &veritech).await?;

        info!("Checking for JWT keys");
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

        Ok(Self {
            pg,
            nats,
            veritech,
            event_log_fs,
            settings,
        })
    }

    async fn run(self) -> anyhow::Result<()> {
        info!("Starting resource scheduler");
        tokio::task::spawn(si_sdf::resource_scheduler::start(
            self.pg.clone(),
            self.nats.clone(),
            self.veritech.clone(),
        ));

        info!("Starting service");
        start(
            self.pg,
            self.nats,
            self.veritech,
            self.event_log_fs,
            self.settings,
        )
        .await;

        Ok(())
    }
}

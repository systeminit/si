use si_data::NatsConn;
use si_settings::Settings;
use si_veritech_2::{start, telemetry};
use tracing::{instrument, log::info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init()?;
    Server::init().await?.run().await
}

struct Server {
    nats: NatsConn,
    settings: Settings,
}

impl Server {
    #[instrument(name = "veritech.init")]
    async fn init() -> anyhow::Result<Self> {
        let server_name = "si-veritech";
        info!("Starting {}", server_name);

        info!("Loading settings");
        let settings = si_settings::Settings::new()?;

        info!("Initializing connection to NATS");
        let nats = NatsConn::new(&settings.nats).await?;

        Ok(Self { nats, settings })
    }

    async fn run(self) -> anyhow::Result<()> {
        info!("Starting service");
        start(self.nats, self.settings).await;

        Ok(())
    }
}

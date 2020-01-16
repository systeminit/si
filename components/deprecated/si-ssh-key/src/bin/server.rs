use si_settings::Settings;
use tokio::runtime::Builder;
use tonic::transport::Server;
use tracing::{span, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use si_ssh_key::{
    data::{self, Db},
    error,
    service::Service,
    ssh_key::server::SshKeyServer,
};

fn main() -> error::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

    let runtime = Builder::new()
        .panic_handler(|err| std::panic::resume_unwind(err))
        .build()?;
    runtime.block_on(async {
        let service_start_span = span!(Level::INFO, "service_init");
        let entered_span = service_start_span.enter();

        let settings = Settings::new()?;

        let db = Db::new(&settings)?;

        let mut data = data::migration_data();
        for d in data.iter_mut() {
            db.migrate_component(d).await?;
        }

        let service = Service::new(db)?;

        drop(entered_span);

        let addr = "[::1]:50052".parse().unwrap();

        Server::builder()
            .add_service(SshKeyServer::new(service))
            .serve(addr)
            .await
            .map_err(error::Error::TonicError)
    })
}

use crate::error::CeaResult;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub mod prelude {
    pub use crate::agent::dispatch::Dispatcher;
    pub use crate::agent::finalizer::AgentFinalizer;
    pub use crate::agent::server::AgentServer;
    pub use si_data::{Db, Storable};
    pub use si_settings::Settings;
}

pub fn setup_tracing() -> CeaResult<()> {
    let subscriber = FmtSubscriber::builder()
        .without_time()
        .with_ansi(true)
        .compact()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

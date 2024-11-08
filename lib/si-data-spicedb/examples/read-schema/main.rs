use std::{env, error, io, str::FromStr};

use si_data_spicedb::{Client, SpiceDbConfig};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
    EnvFilter, Registry,
};
use url::Url;

const TRACING_LOG_ENV_VAR: &str = "SI_LOG";
const DEFAULT_TRACING_DIRECTIVES: &str = "read_schema=trace,si_data_spicedb=trace,info";

#[allow(clippy::disallowed_methods)] // env vars are supporting alternatives in an example
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    Registry::default()
        .with(
            EnvFilter::try_from_env(TRACING_LOG_ENV_VAR)
                .unwrap_or_else(|_| EnvFilter::new(DEFAULT_TRACING_DIRECTIVES)),
        )
        .with(
            fmt::layer()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(io::stderr)
                .pretty(),
        )
        .try_init()?;

    let endpoint = env::var("SPICEDB_URL").unwrap_or_else(|_| "http://localhost:50051".to_owned());
    let preshared_key = env::var("SPICEDB_PRESHARED_KEY")
        .expect("required environment variable: SPICEDB_PRESHARED_KEY");

    let config = SpiceDbConfig {
        enabled: true,
        endpoint: Url::from_str(&endpoint)?,
        preshared_key: preshared_key.into(),
    };

    let mut client = Client::new(&config).await?;

    let r = client.read_schema().await?;
    println!("{}", r.schema_text);

    Ok(())
}

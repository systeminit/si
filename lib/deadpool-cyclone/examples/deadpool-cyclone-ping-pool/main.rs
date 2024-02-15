use deadpool_cyclone::CycloneClient;
use deadpool_cyclone::PoolNoodle;
use std::{
    env,
    path::Path,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use buck2_resources::Buck2Resources;
use deadpool_cyclone::instance::{
    cyclone::{LocalUdsInstance, LocalUdsInstanceSpec},
    Instance,
};
use futures::{stream, StreamExt, TryStreamExt};
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    Registry::default()
        .with(
            EnvFilter::try_from_env("SI_LOG")
                .unwrap_or_else(|_| EnvFilter::new("debug,deadpool_cyclone=trace")),
        )
        .with(fmt::layer())
        .try_init()?;

    let concurrency = match env::args().nth(1) {
        Some(arg) => usize::from_str(&arg)?,
        None => 4,
    };

    let spec = spec()?;
    let mut pool: PoolNoodle<LocalUdsInstance, _> = PoolNoodle::new(10, spec.clone());
    pool.start();

    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    let pings = AtomicU64::new(0);

    let concurrent_pings = stream::repeat_with(|| ping(pool.clone()))
        .map(Ok)
        .try_for_each_concurrent(concurrency, |ping| async {
            let result = ping.await;
            pings.fetch_add(1, Ordering::SeqCst);
            result
        });

    tokio::select! {
        _ = &mut ctrl_c => {
            info!("received ctrl-c signal, shutting down");
        }
        result = concurrent_pings => {
            match result {
                Ok(_) => {
                    info!("finished pings");
                }
                Err(err) => {
                    error!(error = ?err, "found error in ping stream");
                }
            }
        }
    }

    info!("program complete; pings={}", pings.load(Ordering::Relaxed));
    Ok(())
}

async fn ping(
    mut pool: PoolNoodle<LocalUdsInstance, LocalUdsInstanceSpec>,
) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    info!("Getting an instance from the pool");
    let mut instance = pool.get().await?;
    info!("Checking if instance is healthy");
    instance.ensure_healthy().await?;
    info!("Executing ping");
    instance.execute_ping().await?.start().await?;
    Ok(())
}

#[allow(clippy::disallowed_methods)] // Used to determine if running in development
fn spec() -> Result<LocalUdsInstanceSpec, Box<(dyn std::error::Error + 'static)>> {
    if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
        let resources = Buck2Resources::read()?;
        let cyclone_cmd_path = resources
            .get_ends_with("cyclone")?
            .to_string_lossy()
            .to_string();
        let cyclone_decryption_key_path = resources
            .get_ends_with("dev.decryption.key")?
            .to_string_lossy()
            .to_string();
        let lang_server_cmd_path = resources
            .get_ends_with("lang-js")?
            .to_string_lossy()
            .to_string();

        LocalUdsInstance::spec()
            .try_cyclone_cmd_path(cyclone_cmd_path)?
            .cyclone_decryption_key_path(cyclone_decryption_key_path)
            .try_lang_server_cmd_path(lang_server_cmd_path)?
            .ping()
            .build()
            .map_err(Into::into)
    } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        let cyclone_cmd_path = Path::new(&dir)
            .join("../../target/debug/cyclone")
            .canonicalize()
            .expect("failed to canonicalize local dev build of <root>/target/debug/cyclone")
            .to_string_lossy()
            .to_string();
        let cyclone_decryption_key_path = Path::new(&dir)
        .join("../../lib/cyclone-server/src/dev.decryption.key")
        .canonicalize()
        .expect(
            "failed to canonicalize local key at <root>/lib/cyclone-server/src/dev.decryption.key",
        )
        .to_string_lossy()
        .to_string();
        let lang_server_cmd_path = Path::new(&dir)
            .join("../../bin/lang-js/target/lang-js")
            .canonicalize()
            .expect("failed to canonicalize local dev build of <root>/bin/lang-js/target/lang-js")
            .to_string_lossy()
            .to_string();

        LocalUdsInstance::spec()
            .try_cyclone_cmd_path(cyclone_cmd_path)?
            .cyclone_decryption_key_path(cyclone_decryption_key_path)
            .try_lang_server_cmd_path(lang_server_cmd_path)?
            .ping()
            .build()
            .map_err(Into::into)
    } else {
        unimplemented!("not running with Buck2 or Cargo, unsupported")
    }
}

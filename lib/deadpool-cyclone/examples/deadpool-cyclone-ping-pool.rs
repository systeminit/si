use std::{
    env,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use deadpool_cyclone::{
    instance::{
        cyclone::{LocalUdsInstance, LocalUdsInstanceSpec},
        Instance,
    },
    CycloneClient, Manager, Pool,
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

    let spec = LocalUdsInstance::spec()
        .try_cyclone_cmd_path("../../target/debug/cyclone")?
        .cyclone_decryption_key_path("../../lib/cyclone-server/src/dev.decryption.key")
        .try_lang_server_cmd_path("../../bin/lang-js/target/lang-js")?
        .ping()
        .build()?;
    let manager = Manager::new(spec);
    let pool = Pool::builder(manager).max_size(64).build()?;

    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    let pings = AtomicU64::new(0);

    let concurrent_pings = stream::repeat_with(|| ping(&pool))
        .map(Ok)
        .try_for_each_concurrent(concurrency, |ping| async {
            let result = ping.await;
            pings.fetch_add(1, Ordering::SeqCst);
            result
        });

    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                info!("received ctrl-c signal, shutting down");
                break
            }
            result = concurrent_pings => {
                match result {
                    Ok(_) => {
                        info!("finished pings");
                        break
                    }
                    Err(err) => {
                        error!(error = ?err, "found error in ping stream");
                        break
                    }
                }
            }
        }
    }

    info!("closing the pool");
    pool.close();

    info!("program complete; pings={}", pings.load(Ordering::Relaxed));
    Ok(())
}

async fn ping(
    pool: &Pool<LocalUdsInstanceSpec>,
) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    info!("Getting an instance from the pool");
    let mut instance = pool.get().await?;
    info!("Checking if instance is healthy");
    instance.ensure_healthy().await?;
    info!("Executing ping");
    instance.execute_ping().await?.start().await?;
    Ok(())
}

use std::{
    env, io,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use deadpool_cyclone::{
    instance::{
        cyclone::{LocalUdsInstance, LocalUdsInstanceSpec},
        Instance,
    },
    CycloneClient, FunctionResult, Manager, Pool, ProgressMessage, ResolverFunctionRequest,
};
use futures::{stream, StreamExt, TryStreamExt};
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    Registry::default()
        .with(
            EnvFilter::try_from_env("SI_LOG")
                .unwrap_or_else(|_| EnvFilter::new("info,deadpool_cyclone=trace,cyclone=trace")),
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
        .resolver()
        .build()?;
    let manager = Manager::new(spec);
    let pool = Pool::builder(manager).max_size(64).build()?;

    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    info!("waiting for request on stdin...");
    let request: ResolverFunctionRequest = serde_json::from_reader(io::stdin())?;
    info!(request = ?request);
    info!("running executions");

    let count = AtomicU64::new(0);

    let concurrent_executions = stream::repeat_with(|| execute(&pool, &request))
        .map(Ok)
        .try_for_each_concurrent(concurrency, |execution_task| async {
            let result = execution_task.await;
            count.fetch_add(1, Ordering::SeqCst);
            result
        });

    loop {
        tokio::select! {
            _ = &mut ctrl_c => {
                info!("received ctrl-c signal, shutting down");
                break
            }
            result = concurrent_executions => {
                match result {
                    Ok(_) => {
                        info!("finished executions");
                        break
                    }
                    Err(err) => {
                        error!(error = ?err, "found error in execution stream");
                        break
                    }
                }
            }
        }
    }

    info!("closing the pool");
    pool.close();

    info!(
        "program complete; executions={}",
        count.load(Ordering::Relaxed)
    );
    Ok(())
}

async fn execute(
    pool: &Pool<LocalUdsInstanceSpec>,
    request: &ResolverFunctionRequest,
) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    // Generate a random execution_id
    let mut request = (*request).clone();
    request.execution_id = Uuid::new_v4().to_string();

    info!(status = ?pool.status(), "Getting an instance from the pool");
    let mut instance = pool.get().await?;
    info!("Checking if instance is healthy");
    instance.ensure_healthy().await?;

    info!(
        execution_id = &request.execution_id.as_str(),
        "Executing resolver function"
    );
    let mut progress = instance.execute_resolver(request).await?.start().await?;
    while let Some(message) = progress.try_next().await? {
        match message {
            ProgressMessage::Heartbeat => info!("heartbeat"),
            ProgressMessage::OutputStream(output) => {
                info!(
                    execution_id = &output.execution_id.as_str(),
                    stream = &output.stream.as_str(),
                    level = &output.level.as_str(),
                    message = &output.message.as_str(),
                    timestamp = output.timestamp,
                );
            }
        }
    }
    let result = progress.finish().await?;
    match result {
        FunctionResult::Success(success) => info!(
            execution_id = &success.execution_id.as_str(),
            unset = success.unset,
            timestamp = success.timestamp,
        ),
        FunctionResult::Failure(failure) => error!(
            execution_id = &failure.execution_id.as_str(),
            error_kind = &failure.error.kind.as_str(),
            error_message = &failure.error.message.as_str(),
            timestamp = failure.timestamp,
        ),
    }

    Ok(())
}

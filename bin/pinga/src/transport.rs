use futures::StreamExt;
use si_data_nats::NatsClient;
use telemetry_application::prelude::*;
use tokio::sync::watch;

use crate::{config, ExecutionState, JobError};
use dal::{job::consumer::JobInfo, DalContextBuilder};

// pub type Client = faktory_async::Client;
// pub type Subscription = Client;
// pub type Processor = dal::FaktoryProcessor;
// pub async fn connect(config: &config::Config) -> Result<Client, JobError> {
//     Ok(faktory_connect(config))
// }
// pub async fn subscribe(client: &Client) -> Result<Subscription, JobError> {
//     Ok(client.clone())
// }
// pub async fn fetch(
//     subscription: &Subscription,
//     shutdown_request_rx: &mut watch::Receiver<()>,
//     ctx_builder: &DalContextBuilder,
// ) -> ExecutionState {
//     faktory_fetch(subscription, shutdown_request_rx, ctx_builder).await
// }
// pub async fn post_process(subscription: &Subscription, jid: String, result: Result<(), JobError>) {
//     faktory_post_process(subscription, jid, result).await
// }

pub type Client = NatsClient;
pub type Subscription = si_data_nats::Subscription;
pub type Processor = dal::NatsProcessor;
pub async fn connect(config: &config::Config) -> Result<Client, JobError> {
    nats_connect(config).await
}
pub async fn subscribe(client: &Client) -> Result<Subscription, JobError> {
    nats_subscribe(client).await
}
pub async fn fetch(
    subscription: &mut Subscription,
    shutdown_request_rx: &mut watch::Receiver<()>,
    ctx_builder: &DalContextBuilder,
) -> ExecutionState {
    nats_fetch(subscription, shutdown_request_rx, ctx_builder).await
}
pub async fn post_process(subscription: &Subscription, jid: String, result: Result<(), JobError>) {
    nats_post_process(subscription, jid, result).await
}

// ---------------------------------
// Transport specific implementation
// ---------------------------------

#[allow(unused)]
async fn nats_connect(config: &config::Config) -> Result<NatsClient, JobError> {
    Ok(NatsClient::new(config.nats()).await?)
}
#[allow(unused)]
pub async fn nats_subscribe(client: &NatsClient) -> Result<si_data_nats::Subscription, JobError> {
    Ok(client.queue_subscribe("pinga-jobs", "pinga").await?)
}
#[allow(unused)]
async fn nats_fetch(
    subscription: &mut si_data_nats::Subscription,
    shutdown_request_rx: &mut watch::Receiver<()>,
    ctx_builder: &DalContextBuilder,
) -> ExecutionState {
    tokio::select! {
        job = subscription.next() => match job {
            None => ExecutionState::Idle,
            Some(result) => match result {
                Ok(msg) => match serde_json::from_slice::<JobInfo>(msg.data()) {
                    Ok(job) => ExecutionState::Process(job),
                    Err(err) => {
                        error!("Unable to deserialize nats' job: {err}");
                        ExecutionState::Idle
                    }
                }
                Err(err) => {
                    error!("Internal error in nats, bailing out: {err}");
                    ExecutionState::Stop
                }
            }
        },
        _ = shutdown_request_rx.changed() => {
            info!("Worker task received shutdown notification: stopping");
            ExecutionState::Stop
        }
    }
}

#[allow(unused)]
async fn nats_post_process(
    client: &si_data_nats::Subscription,
    jid: String,
    result: Result<(), JobError>,
) {
    match result {
        Ok(()) => {}
        Err(err) => error!("Job execution failed: {err}"),
    }
}

#[allow(unused)]
fn faktory_connect(config: &config::Config) -> faktory_async::Client {
    let config = faktory_async::Config::from_uri(
        &config.faktory().url,
        Some("pinga".to_string()),
        Some(uuid::Uuid::new_v4().to_string()),
    );
    faktory_async::Client::new(config, 256)
}
#[allow(unused)]
async fn faktory_fetch(
    client: &faktory_async::Client,
    shutdown_request_rx: &mut watch::Receiver<()>,
    ctx_builder: &DalContextBuilder,
) -> ExecutionState {
    use faktory_async::BeatState;

    match client.last_beat().await {
        Ok(BeatState::Ok) => {
            tokio::select! {
                job = client.fetch(vec!["default".into()]) => match job {
                    Ok(Some(job)) => match JobInfo::try_from(job) {
                        Ok(job) => ExecutionState::Process(job),
                        Err(err) => {
                            error!("Unable to deserialize faktory's job: {err}");
                            ExecutionState::Idle
                        }
                    },
                    Ok(None) => ExecutionState::Idle,
                    Err(err) => {
                        error!("Unable to fetch from faktory: {err}");
                        ExecutionState::Idle
                    }
                },
                _ = shutdown_request_rx.changed() => {
                    info!("Worker task received shutdown notification: stopping");
                    ExecutionState::Stop
                }
            }
        }
        Ok(BeatState::Quiet) => {
            // Getting a "quiet" state from the faktory server means that
            // someone has gone to the faktory UI and requested that this
            // particular worker finish what it's doing, and gracefully
            // shut down.
            info!("Gracefully shutting down from Faktory request.");
            ExecutionState::Stop
        }
        Ok(BeatState::Terminate) => {
            warn!("Faktory asked us to terminate");
            ExecutionState::Stop
        }
        Err(err) => {
            error!("Internal error in faktory-async, bailing out: {err}");
            ExecutionState::Stop
        }
    }
}

#[allow(unused)]
async fn faktory_post_process(
    client: &faktory_async::Client,
    jid: String,
    result: Result<(), JobError>,
) {
    use faktory_async::FailConfig;
    match result {
        Ok(()) => match client.ack(jid).await {
            Ok(()) => {}
            Err(err) => {
                error!("Ack failed: {err}");
            }
        },
        Err(err) => {
            error!("Job execution failed: {err}");
            // TODO: pass backtrace here
            match client
                .fail(FailConfig::new(
                    jid,
                    format!("{err:?}"),
                    err.to_string(),
                    None,
                ))
                .await
            {
                Ok(()) => {}
                Err(err) => error!("Fail failed: {err}"),
            }
        }
    }
}

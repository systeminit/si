use std::{
    hash::Hash,
    time::Duration,
};

use async_nats::jetstream::consumer::pull::Config as ConsumerConfig;
use futures::StreamExt as _;
use telemetry_utils::monotonic;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{
    debug,
    error,
    info,
    warn,
};

use super::{
    config::FairSchedulingConfig,
    stream::KeyReady,
};

const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);
const BASE_RETRY_DELAY: Duration = Duration::from_secs(1);

enum ConsumerCreationResult {
    Success,
    ChannelClosed,
    CreationFailed,
}

#[derive(Debug, Error)]
pub enum TaskListenerError {
    #[error("failed to create task listener consumer: {0}")]
    CreateConsumer(#[source] async_nats::jetstream::stream::ConsumerError),
    #[error("failed to get messages stream: {0}")]
    Messages(#[source] async_nats::jetstream::consumer::StreamError),
    #[error("failed to get/create key consumer: {0}")]
    GetKeyConsumer(#[source] async_nats::jetstream::stream::ConsumerError),
    #[error("failed to get key consumer messages: {0}")]
    KeyConsumerMessages(#[source] async_nats::jetstream::consumer::StreamError),
}

pub fn spawn_task_listener<K>(
    config: FairSchedulingConfig<K>,
    consumer_tx: mpsc::Sender<KeyReady<K>>,
    shutdown: CancellationToken,
) -> tokio::task::JoinHandle<()>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
{
    tokio::spawn(async move {
        let mut retry_count = 0u32;

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => {
                    debug!("task listener shutdown");
                    break;
                }
                result = run_task_listener(&config, consumer_tx.clone(), shutdown.clone()) => {
                    match result {
                        Ok(()) => {
                            debug!("task listener completed");
                            break;
                        }
                        Err(e) => {
                            retry_count += 1;
                            let delay = std::cmp::min(
                                BASE_RETRY_DELAY * 2u32.saturating_pow(retry_count.saturating_sub(1)),
                                MAX_RETRY_DELAY,
                            );
                            error!(error = %e, retry_count, "task listener error, retrying after {}s", delay.as_secs());
                            monotonic!(naxum.fair_scheduler.task_listener_restarts = 1);
                            tokio::time::sleep(delay).await;
                        }
                    }
                }
            }
        }
    })
}

async fn run_task_listener<K>(
    config: &FairSchedulingConfig<K>,
    consumer_tx: mpsc::Sender<KeyReady<K>>,
    shutdown: CancellationToken,
) -> Result<(), TaskListenerError>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
{
    let consumer = config
        .tasks_stream
        .create_consumer(ConsumerConfig {
            durable_name: Some(config.task_listener_name.clone()),
            filter_subject: config.tasks_filter_subject.clone(),
            ..Default::default()
        })
        .await
        .map_err(TaskListenerError::CreateConsumer)?;

    let mut messages = consumer
        .messages()
        .await
        .map_err(TaskListenerError::Messages)?;

    loop {
        tokio::select! {
            _ = shutdown.cancelled() => {
                debug!("task listener shutting down");
                break;
            }
            msg = messages.next() => {
                let Some(msg) = msg else {
                    debug!("task stream ended");
                    break;
                };

                let msg = match msg {
                    Ok(m) => m,
                    Err(e) => {
                        warn!(error = %e, "error receiving task message");
                        continue;
                    }
                };

                let subject = msg.subject.as_str();
                monotonic!(naxum.fair_scheduler.task_notifications = 1);

                let Some(key) = (config.key_extractor)(subject, config.subject_prefix.as_deref(), &config.service_name) else {
                    warn!(%subject, "could not extract key");
                    let _ = msg.ack().await; // Best effort ack for invalid message, ignore errors
                    continue;
                };

                // Always create/refresh the consumer for this key. The FairSchedulerStream
                // handles deduplication - if a consumer already exists for this key, the new
                // one replaces it. This ensures work is never missed when a workspace becomes
                // active again after its previous consumer was exhausted.
                let result = create_and_send_consumer(config, &key, &consumer_tx).await;

                // Ack on success or channel closed; don't ack on failure (let NATS redeliver)
                let should_ack = !matches!(result, ConsumerCreationResult::CreationFailed);

                if should_ack {
                    if let Err(e) = msg.ack().await {
                        warn!(error = %e, "failed to ack task message");
                        monotonic!(naxum.fair_scheduler.task_ack_errors = 1);
                    }
                }

                // If channel closed, break the loop
                if matches!(result, ConsumerCreationResult::ChannelClosed) {
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn create_and_send_consumer<K>(
    config: &FairSchedulingConfig<K>,
    key: &K,
    consumer_tx: &mpsc::Sender<KeyReady<K>>,
) -> ConsumerCreationResult
where
    K: Clone + Eq + Hash,
{
    let consumer_name = (config.consumer_name_fn)(key, &config.service_name);
    let filter_subject = (config.consumer_filter_subject_fn)(
        config.subject_prefix.as_deref(),
        key,
        &config.service_name,
    );

    debug!(%consumer_name, %filter_subject, "creating key consumer");

    let consumer = match config
        .requests_stream
        .get_or_create_consumer(
            &consumer_name,
            ConsumerConfig {
                durable_name: Some(consumer_name.clone()),
                filter_subject,
                inactive_threshold: config.inactive_threshold,
                ..Default::default()
            },
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "failed to create key consumer");
            monotonic!(naxum.fair_scheduler.consumer_create_errors = 1);
            return ConsumerCreationResult::CreationFailed;
        }
    };

    let key_messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            warn!(error = %e, "failed to get key consumer messages");
            monotonic!(naxum.fair_scheduler.consumer_create_errors = 1);
            return ConsumerCreationResult::CreationFailed;
        }
    };

    monotonic!(naxum.fair_scheduler.consumers_created = 1);

    if consumer_tx
        .send(KeyReady {
            key: key.clone(),
            messages: key_messages,
        })
        .await
        .is_err()
    {
        debug!("consumer_tx closed");
        return ConsumerCreationResult::ChannelClosed;
    }

    ConsumerCreationResult::Success
}

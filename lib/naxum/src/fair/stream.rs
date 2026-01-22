// Fair scheduler stream for balanced message processing across keys.

use std::{
    collections::HashMap,
    hash::Hash,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use async_nats::jetstream::{
    Message,
    consumer::pull::Stream as MessageStream,
};
use futures::Stream;
use telemetry_utils::{
    counter,
    monotonic,
};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{
    info,
    trace,
};

use super::scheduler::Scheduler;

/// Notification that a key has work available.
pub struct KeyReady<K> {
    pub key: K,
    pub messages: MessageStream,
}

#[derive(Debug, Error)]
pub enum FairSchedulerError {
    #[error("consumer stream error: {0}")]
    ConsumerStream(#[source] async_nats::jetstream::consumer::pull::MessagesError),
}

pub struct FairSchedulerStream<K> {
    scheduler: Scheduler<K>,
    consumers: HashMap<K, Pin<Box<MessageStream>>>,
    consumer_rx: mpsc::Receiver<KeyReady<K>>,
    shutdown: CancellationToken,
}

impl<K> FairSchedulerStream<K>
where
    K: Clone + Eq + Hash,
{
    pub fn new(consumer_rx: mpsc::Receiver<KeyReady<K>>, shutdown: CancellationToken) -> Self {
        Self {
            scheduler: Scheduler::new(),
            consumers: HashMap::new(),
            consumer_rx,
            shutdown,
        }
    }

    fn drain_new_consumers(&mut self) {
        while let Ok(ready) = self.consumer_rx.try_recv() {
            trace!("received key consumer for draining");
            self.insert_consumer(ready.key, ready.messages);
        }
    }

    fn insert_consumer(&mut self, key: K, messages: MessageStream) {
        if let Some(old_consumer) = self.consumers.insert(key.clone(), Box::pin(messages)) {
            // Replacing an existing consumer - clean up the old one
            trace!("replacing existing consumer, notifying has work");
            drop(old_consumer);
            self.scheduler.notify_has_work(key.clone());
        } else {
            // New consumer - increment the metric and notify scheduler
            trace!("inserting new consumer, notifying has work");
            counter!(naxum.fair_scheduler.active_keys = 1);
            self.scheduler.notify_has_work(key);
        }
    }

    fn cleanup(&mut self, key: K) {
        self.scheduler.notify_no_work(&key);
        self.consumers.remove(&key);
        counter!(naxum.fair_scheduler.active_keys = -1);
        monotonic!(naxum.fair_scheduler.key_empty = 1);
    }
}

impl<K> Stream for FairSchedulerStream<K>
where
    K: Clone + Eq + Hash + Unpin,
{
    type Item = Result<Message, FairSchedulerError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.shutdown.is_cancelled() {
            return Poll::Ready(None);
        }

        loop {
            // Drain all available consumers from the channel
            this.drain_new_consumers();

            if let Some(key) = this.scheduler.next() {
                let Some(consumer) = this.consumers.get_mut(&key) else {
                    this.scheduler.notify_no_work(&key);
                    continue;
                };

                trace!("scheduler selected key, polling consumer");
                match consumer.as_mut().poll_next(cx) {
                    Poll::Ready(Some(Ok(msg))) => {
                        // Got message - update state and return
                        this.scheduler.complete(&key, 1);
                        monotonic!(naxum.fair_scheduler.messages_processed = 1);
                        return Poll::Ready(Some(Ok(msg)));
                    }
                    Poll::Ready(Some(Err(e))) => {
                        return Poll::Ready(Some(Err(FairSchedulerError::ConsumerStream(e))));
                    }
                    Poll::Ready(None) => {
                        // Consumer exhausted - clean up and try next
                        trace!("consumer exhausted, marking no work");
                        this.cleanup(key);
                        continue;
                    }
                    Poll::Pending => {
                        // Consumer pending but not idle yet - wake ourselves to retry
                        // This ensures we keep polling even if the JetStream consumer
                        // doesn't properly propagate wakeups
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                }
            } else {
                // No keys with work - wait for new consumers on the channel
                trace!("no keys with work, waiting for consumer notifications");
                match Pin::new(&mut this.consumer_rx).poll_recv(cx) {
                    Poll::Ready(Some(ready)) => {
                        trace!("received new consumer notification");
                        this.insert_consumer(ready.key, ready.messages);
                        // Loop to try polling the new consumer
                        continue;
                    }
                    Poll::Ready(None) => {
                        trace!("consumer channel closed, terminating stream");
                        return Poll::Ready(None);
                    }
                    Poll::Pending => {
                        trace!("no new consumers, returning pending");
                        return Poll::Pending;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::StreamExt;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    use super::super::FairSchedulerStream;

    #[tokio::test]
    async fn test_fair_stream_shutdown_cancels_immediately() {
        let (_consumer_tx, consumer_rx) = mpsc::channel(10);
        let shutdown = CancellationToken::new();

        let mut stream = FairSchedulerStream::<String>::new(consumer_rx, shutdown.clone());

        // Cancel immediately
        shutdown.cancel();

        // Stream should return None (terminated)
        let result = stream.next().await;
        assert!(result.is_none(), "Stream should terminate on shutdown");
    }

    #[tokio::test]
    async fn test_fair_stream_handles_no_consumers() {
        let (_consumer_tx, consumer_rx) = mpsc::channel(10);
        let shutdown = CancellationToken::new();

        let mut stream = FairSchedulerStream::<String>::new(consumer_rx, shutdown.clone());

        // Try to poll with no consumers (should be pending)
        tokio::select! {
            _ = stream.next() => {
                panic!("Stream should not yield without consumers");
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Expected: stream is waiting for consumers
            }
        }

        shutdown.cancel();
    }

    #[tokio::test]
    async fn test_fair_stream_returns_none_when_consumer_channel_closes() {
        let (consumer_tx, consumer_rx) = mpsc::channel(10);
        let shutdown = CancellationToken::new();

        let mut stream = FairSchedulerStream::<String>::new(consumer_rx, shutdown.clone());

        // Close the consumer channel (no consumers will ever arrive)
        drop(consumer_tx);

        // Stream should return None when channel is closed
        let result = stream.next().await;
        assert!(
            result.is_none(),
            "Stream should terminate when consumer channel closes"
        );
    }

    #[tokio::test]
    async fn test_fair_stream_shutdown_vs_channel_close() {
        // Test that shutdown takes precedence over channel operations
        let (consumer_tx, consumer_rx) = mpsc::channel(10);
        let shutdown = CancellationToken::new();

        let mut stream = FairSchedulerStream::<String>::new(consumer_rx, shutdown.clone());

        // Cancel shutdown before closing channel
        shutdown.cancel();

        // Even though channel is still open, shutdown should terminate stream
        let result = stream.next().await;
        assert!(
            result.is_none(),
            "Shutdown should terminate stream immediately"
        );

        // Clean up
        drop(consumer_tx);
    }
}

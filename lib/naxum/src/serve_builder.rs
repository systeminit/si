use std::{
    convert::Infallible,
    error,
    fmt,
    hash::Hash,
    marker::PhantomData,
    sync::Arc,
};

use async_nats::jetstream::Message as JsMessage;
use futures::Stream;
use tokio::sync::{
    Semaphore,
    mpsc::{
        self,
        Receiver,
        Sender,
    },
};
use tokio_util::sync::CancellationToken;

use crate::{
    fair::{
        FairSchedulerError,
        FairSchedulerStream,
        FairSchedulingConfig,
        KeyReady,
        spawn_task_listener,
    },
    message::{
        Message,
        MessageHead,
    },
    response::Response,
    serve::{
        IncomingMessage,
        SemaphoreMode,
        Serve,
    },
};

pub struct NoFairScheduling;
pub struct WithFairScheduling<K> {
    config: FairSchedulingConfig<K>,
}

/// Builder for configuring the naxum serve loop.
///
/// Uses type-state pattern to ensure correct configuration at compile time.
///
/// # Example
///
/// ```ignore
/// // Simple case - no fair scheduling
/// ServeBuilder::new(app)
///     .with_incoming_limit(100)
///     .serve(stream)
///     .with_graceful_shutdown(shutdown_signal)
///     .await?;
///
/// // With fair scheduling
/// ServeBuilder::new(app)
///     .with_external_semaphore(semaphore)
///     .with_fair_scheduling(config)
///     .serve()
///     .with_graceful_shutdown(shutdown_signal)
///     .await?;
/// ```
pub struct ServeBuilder<M, S, FS = NoFairScheduling> {
    make_service: M,
    semaphore: Option<SemaphoreMode>,
    fair_scheduling: FS,
    _marker: PhantomData<S>,
}

impl<M, S, FS> fmt::Debug for ServeBuilder<M, S, FS>
where
    M: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServeBuilder")
            .field("make_service", &self.make_service)
            .finish_non_exhaustive()
    }
}

impl<M, S> ServeBuilder<M, S, NoFairScheduling> {
    /// Creates a new serve builder with the given service.
    pub fn new(make_service: M) -> Self {
        Self {
            make_service,
            semaphore: None,
            fair_scheduling: NoFairScheduling,
            _marker: PhantomData,
        }
    }

    /// Sets a limit on the number of concurrent incoming messages.
    ///
    /// This creates an internal semaphore that limits concurrency.
    /// Permits are released when message processing completes.
    pub fn with_incoming_limit(mut self, limit: usize) -> Self {
        self.semaphore = Some(SemaphoreMode::Internal(Arc::new(Semaphore::new(limit))));
        self
    }

    /// Uses an external semaphore for concurrency control.
    ///
    /// Permits are "forgotten" after acquisition - the semaphore owner
    /// is responsible for restoring permits via `add_permits()`.
    pub fn with_external_semaphore(mut self, semaphore: Arc<Semaphore>) -> Self {
        self.semaphore = Some(SemaphoreMode::External(semaphore));
        self
    }

    /// Serves messages from the given stream.
    ///
    /// Use this for the standard case without fair scheduling.
    pub fn serve<T, E, R>(self, stream: T) -> Serve<M, S, T, E, R>
    where
        M: for<'a> tower::Service<IncomingMessage<'a, R>, Error = Infallible, Response = S>,
        S: tower::Service<Message<R>, Response = Response, Error = Infallible>
            + Clone
            + Send
            + 'static,
        S::Future: Send,
        T: Stream<Item = Result<R, E>>,
        E: error::Error,
        R: MessageHead,
    {
        Serve {
            stream,
            make_service: self.make_service,
            semaphore: self.semaphore,
            task_listener_handle: None,
            shutdown_token: None,
            _service_marker: PhantomData,
            _stream_error_marker: PhantomData,
            _request_marker: PhantomData,
        }
    }

    /// Configures fair scheduling.
    ///
    /// Fair scheduling ensures that no single key (e.g., workspace) can
    /// monopolize processing capacity.
    ///
    /// An internal shutdown token will be created to coordinate between the
    /// fair scheduler stream and task listener. The token will be cancelled
    /// automatically during graceful shutdown.
    pub fn with_fair_scheduling<K>(
        self,
        config: FairSchedulingConfig<K>,
    ) -> ServeBuilder<M, S, WithFairScheduling<K>>
    where
        K: Clone + Eq + Hash + Send + Sync + 'static,
    {
        ServeBuilder {
            make_service: self.make_service,
            semaphore: self.semaphore,
            fair_scheduling: WithFairScheduling { config },
            _marker: PhantomData,
        }
    }
}

impl<M, S, K> ServeBuilder<M, S, WithFairScheduling<K>>
where
    K: Clone + Eq + Hash + Send + Sync + Unpin + 'static,
{
    /// Creates the fair scheduler and returns a Serve instance.
    ///
    /// This spawns the task listener in the background and creates the fair scheduler stream.
    /// An internal shutdown token is created to coordinate between components and will be
    /// cancelled automatically during graceful shutdown.
    pub fn serve(self) -> Serve<M, S, FairSchedulerStream<K>, FairSchedulerError, JsMessage>
    where
        M: for<'a> tower::Service<IncomingMessage<'a, JsMessage>, Error = Infallible, Response = S>,
        S: tower::Service<Message<JsMessage>, Response = Response, Error = Infallible>
            + Clone
            + Send
            + 'static,
        S::Future: Send,
    {
        let WithFairScheduling { config } = self.fair_scheduling;

        let shutdown = CancellationToken::new();

        // Channel for task listener to send key consumers to fair scheduler
        let (consumer_tx, consumer_rx): (Sender<KeyReady<K>>, Receiver<KeyReady<K>>) =
            mpsc::channel(64);

        let task_listener_handle = spawn_task_listener(config, consumer_tx, shutdown.clone());
        let stream = FairSchedulerStream::new(consumer_rx, shutdown.clone());

        Serve {
            stream,
            make_service: self.make_service,
            semaphore: self.semaphore,
            task_listener_handle: Some(task_listener_handle),
            shutdown_token: Some(shutdown),
            _service_marker: PhantomData,
            _stream_error_marker: PhantomData,
            _request_marker: PhantomData,
        }
    }
}

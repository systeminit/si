//! The [`StatusReceiver`] is a spawned, "long-running" [tokio](https://tokio.rs/) task that
//! receives "UDP-style" messages from [`DependentValuesUpdate`](crate::DependentValuesUpdate) jobs
//! over [NATS](https://nats.io) in order to perform arbitrary tasks based on what's been received.

use std::collections::{HashMap, HashSet};
use std::panic::AssertUnwindSafe;

use futures::FutureExt;
use futures::StreamExt;
use nats_subscriber::{Request, SubscriberError, Subscription};
use serde::Deserialize;
use serde::Serialize;
use si_data_nats::NatsError;
use si_data_pg::{PgError, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::broadcast;

use crate::{
    AttributeValueId, Component, DalContextBuilder, FixResolver, ServicesContext,
    StandardModelError, TransactionsError, Visibility, WsEvent,
};

pub mod client;

/// The [NATS](https://nats.io) subject for publishing and subscribing to
/// [`requests`](DependentValuesUpdateRequest).
const STATUS_RECEIVER_REQUEST_SUBJECT: &str = "dependentValuesUpdateStatus.request";
/// The queue name for [NATS](https://nats.io).
const STATUS_RECEIVER_QUEUE_NAME: &str = "dependentValuesUpdateStatus";

#[derive(Error, Debug)]
pub enum StatusReceiverError {
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModelError(#[from] StandardModelError),
    #[error(transparent)]
    Subscriber(#[from] SubscriberError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type StatusReceiverResult<T> = Result<T, StatusReceiverError>;

/// The request payload contents when communicating with the
/// [`receiver`](StatusReceiver).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusReceiverRequest {
    pub visibility: Visibility,
    pub dependent_graph: HashMap<AttributeValueId, Vec<AttributeValueId>>,
}

/// The [`StatusReceiver`] evaluates incremental progress and discrete events
/// from the [`DependentValuesUpdate`](crate::DependentValuesUpdate) and performs arbitrary
/// actions based on the requests received.
#[derive(Debug)]
pub struct StatusReceiver {
    /// The [`ServicesContext`](crate::ServicesContext) needed to assemble a
    /// [`DalContext`](crate::DalContext), connect to [NATS](https://nats.io), connect to the
    /// database and interact with essential services.
    services_context: ServicesContext,
    /// A [NATS](https://nats.io) subscription to listen for [`requests`](StatusReceiverRequest).
    requests: Subscription<StatusReceiverRequest>,
}

impl StatusReceiver {
    /// Create a new [`StatusReceiver`].
    pub async fn new(services_context: ServicesContext) -> StatusReceiverResult<Self> {
        // TODO(nick): add ability to alter nats subject or add prefix.
        let nats = services_context.nats_conn();
        let requests: Subscription<StatusReceiverRequest> = Subscription::new(
            nats,
            STATUS_RECEIVER_REQUEST_SUBJECT,
            Some(STATUS_RECEIVER_QUEUE_NAME),
        )
        .await?;
        Ok(Self {
            services_context,
            requests,
        })
    }

    /// A _synchronous_ function that starts the [`receiver`](Self) in a new asynchronous task.
    pub fn start(self, shutdown_broadcast_rx: broadcast::Receiver<()>) {
        tokio::spawn(Self::start_task(
            self.services_context,
            self.requests,
            shutdown_broadcast_rx,
        ));
    }

    /// The "inner" portion of [`Self::start()`] that contains the core listener loop.
    ///
    /// This should only be called by [`Self::start()`].
    #[instrument(name = "status_receiver.start_task", skip_all, level = "debug")]
    async fn start_task(
        services_context: ServicesContext,
        mut requests: Subscription<StatusReceiverRequest>,
        mut shutdown_broadcast_rx: broadcast::Receiver<()>,
    ) {
        loop {
            tokio::select! {
                _ = shutdown_broadcast_rx.recv() => {
                    trace!("the status receiver task received shutdown");
                    break;
                }
                request = requests.next() => {
                    match request {
                        Some(Ok(request)) => {
                            // Spawn a task and process the request. Use the wrapper to handle
                            // returned errors.
                            tokio::spawn(Self::process_wrapper(services_context.clone().into_builder(), request));
                        }
                        Some(Err(err)) => {
                            warn!(error = ?err, "next status receiver request errored");
                        }
                        None => {
                            trace!("status receiver requests subscriber stream has closed");
                            break;
                        }
                    }
                }
                else => {
                    trace!("returning with all select arms closed");
                    break
                }
            }
        }

        // Unsubscribe from subscription.
        if let Err(e) = requests.unsubscribe().await {
            error!("could not unsubscribe from nats: {:?}", e);
        }
    }

    /// A wrapper around [`Self::process()`] to handle returned errors.
    async fn process_wrapper(
        ctx_builder: DalContextBuilder,
        request: Request<StatusReceiverRequest>,
    ) {
        match AssertUnwindSafe(Self::process(ctx_builder, request))
            .catch_unwind()
            .await
        {
            Ok(Ok(())) => {}
            Ok(Err(err)) => error!("{err}"),
            Err(any) => {
                // Technically, panics can be of any shape, but most should be of type "&str" or
                // "String".
                match any.downcast::<String>() {
                    Ok(msg) => error!("panic: {msg}"),
                    Err(any) => match any.downcast::<&str>() {
                        Ok(msg) => error!("panic: {msg}"),
                        Err(any) => {
                            let id = any.type_id();
                            error!("panic message downcast failed of {id:?}",);
                        }
                    },
                }
            }
        }
    }

    /// Evaluate the request from the [`DependentValuesUpdate`](crate::DependentValuesUpdate) job
    /// and perform work accordingly.
    ///
    /// This function is considered the "critical section" of the [`receiver`](Self).
    async fn process(
        ctx_builder: DalContextBuilder,
        request: Request<StatusReceiverRequest>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut ctx = ctx_builder.build_default().await?;

        // Ensure we have the correct visibility before we do anything else.
        ctx.update_visibility(request.payload.visibility);

        let confirmation_attribute_values: HashSet<AttributeValueId> = HashSet::from_iter(
            Component::list_confirmations(&ctx)
                .await?
                .into_iter()
                .map(|cv| cv.attribute_value_id),
        );

        let mut flattened_dependent_graph: Vec<&AttributeValueId> =
            request.payload.dependent_graph.keys().collect();
        flattened_dependent_graph.extend(request.payload.dependent_graph.values().flatten());

        for dependent_value in flattened_dependent_graph {
            if confirmation_attribute_values.contains(dependent_value) {
                WsEvent::confirmations_updated(&ctx)
                    .await?
                    .publish(&ctx)
                    .await?;
                break;
            }
        }

        for confirmation_attribute_value in confirmation_attribute_values {
            let resolver = FixResolver::find_for_confirmation_attribute_value(
                &ctx,
                confirmation_attribute_value,
            )
            .await?;
            if let Some(mut resolver) = resolver {
                resolver.set_success(&ctx, None::<bool>).await?;
            }
        }

        Ok(())
    }
}

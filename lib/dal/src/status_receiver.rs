//! The [`StatusReceiver`] is a spawned, "long-running" [tokio](https://tokio.rs/) task that
//! receives "UDP-style" messages from [`DependentValuesUpdate`](crate::DependentValuesUpdate) jobs
//! over [NATS](https://nats.io) in order to perform arbitrary tasks based on what's been received.

use std::collections::{HashMap, HashSet};
use std::panic::AssertUnwindSafe;

use futures::FutureExt;
use futures::StreamExt;
use nats_subscriber::{Request, Subscription};
use serde::Deserialize;
use serde::Serialize;
use si_data_nats::NatsError;
use si_data_pg::{PgError, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::broadcast;

use crate::{
    AttributeValueId, Component, DalContext, FixResolver, ServicesContext, StandardModelError,
    TransactionsError, Visibility, WsEvent,
};

pub mod client;

/// The [NATS](https://nats.io) subject for publishing and subscribing to
/// [`requests`](DependentValuesUpdateRequest).
const STATUS_RECEIVER_REQUEST_SUBJECT: &str = "dependentValuesUpdateStatus.request";

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
#[derive(Debug, Clone)]
pub struct StatusReceiver {
    /// The [`ServicesContext`](crate::ServicesContext) needed to assemble a
    /// [`DalContext`](crate::DalContext), connect to [NATS](https://nats.io), connect to the
    /// database and interact with essential services.
    services_context: ServicesContext,
}

impl StatusReceiver {
    /// Create a new [`StatusReceiver`].
    pub fn new(services_context: ServicesContext) -> StatusReceiver {
        // TODO(nick): add ability to alter nats subject or add prefix.
        StatusReceiver { services_context }
    }

    /// A _synchronous_ function that starts the [`receiver`](Self) in a new asynchronous task.
    pub fn start(self, shutdown_broadcast_rx: broadcast::Receiver<()>) {
        tokio::spawn(async move {
            tokio::select! {
                _ = self.start_task(shutdown_broadcast_rx) => {}
            }
            info!("status receiver stopped");
        });
    }

    /// The "inner" portion of [`Self::start()`] that contains the core listener loop.
    #[instrument(name = "status_receiver.start_task", skip_all, level = "debug")]
    async fn start_task(&self, mut shutdown_broadcast_rx: broadcast::Receiver<()>) {
        let nats = self.services_context.nats_conn();

        // FIXME(nick): we should not panic here if we cannot subscribe... or should we?
        let mut requests: Subscription<StatusReceiverRequest> =
            Subscription::new(nats, STATUS_RECEIVER_REQUEST_SUBJECT)
                .await
                .expect("could not subscribe to nats");

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
                            let builder = self.services_context.clone().into_builder();
                            match builder.build_default().await {
                                Ok(ctx) => {
                                    tokio::spawn(Self::process_wrapper(ctx, request));
                                }
                                Err(e) => {
                                    error!("could not build dal context: {:?}", e);
                                }
                            }
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
    async fn process_wrapper(ctx: DalContext, request: Request<StatusReceiverRequest>) {
        match AssertUnwindSafe(Self::process(ctx, request))
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
        mut ctx: DalContext,
        request: Request<StatusReceiverRequest>,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

//! The [`StatusReceiver`] is a spawned, "long-running" [tokio](https://tokio.rs/) task that
//! receives "UDP-style" messages from [`DependentValuesUpdate`](crate::DependentValuesUpdate) jobs
//! over [NATS](https://nats.io) in order to perform arbitrary tasks based on what's been received.

use std::collections::{HashMap, HashSet};

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
    AttributeValue, AttributeValueError, AttributeValueId, Component, ComponentId, DalContext,
    DalContextBuilder, ServicesContext, StandardModel, StandardModelError, Tenancy,
    TransactionsError, Visibility, WsEvent,
};

pub mod client;

/// The [NATS](https://nats.io) subject for publishing and subscribing to
/// [`requests`](StatusReceiverRequest).
const STATUS_RECEIVER_REQUEST_SUBJECT: &str = "dependentValuesUpdateStatus.request";
/// The queue name for [NATS](https://nats.io).
const STATUS_RECEIVER_QUEUE_NAME: &str = "dependentValuesUpdateStatus";

#[remain::sorted]
#[derive(Error, Debug)]
pub enum StatusReceiverError {
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
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
    pub tenancy: Tenancy,
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
        let requests: Subscription<StatusReceiverRequest> =
            Subscription::create(STATUS_RECEIVER_REQUEST_SUBJECT)
                .queue_name(STATUS_RECEIVER_QUEUE_NAME)
                .start(nats)
                .await?;
        Ok(Self {
            services_context,
            requests,
        })
    }

    /// A _synchronous_ function that starts the [`receiver`](Self) in a new asynchronous task.
    pub fn start(self, shutdown_broadcast_rx: broadcast::Receiver<()>) {
        info!("starting status receiver for dependent values update jobs");
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
                            // Spawn a task and process the request.
                            let context_clone = services_context.clone();
                            tokio::spawn(
                                async move {
                                    Self::process(context_clone.into_builder(false), request).await.expect("Task failed successfully")
                                }
                            );
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

        // Unsubscribe from subscription without draining the channel
        if let Err(e) = requests.unsubscribe_after(0).await {
            error!("could not unsubscribe from nats: {:?}", e);
        }
    }

    /// Evaluate the request from the [`DependentValuesUpdate`](crate::DependentValuesUpdate) job
    /// and perform work accordingly.
    ///
    /// This function is considered the "critical section" of the [`receiver`](Self). It _CANNOT_
    /// mutate rows that [`DependentValuesUpdate`](crate::DependentValuesUpdate) is mutating,
    /// otherwise a database deadlock may occur.
    async fn process(
        ctx_builder: DalContextBuilder,
        request: Request<StatusReceiverRequest>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("processing request from dependent values update job");
        let mut ctx = ctx_builder.build_default().await?;

        // Ensure we have the correct visibility and tenancy before we do anything else.
        ctx.update_visibility(request.payload.visibility);
        ctx.update_tenancy(request.payload.tenancy);

        let code_generation_attribute_values: HashSet<AttributeValueId> =
            Component::all_code_generation_attribute_values(&ctx).await?;
        let (confirmation_views, _) = Component::list_confirmations(&ctx).await?;
        let confirmation_attribute_values: HashSet<AttributeValueId> =
            HashSet::from_iter(confirmation_views.iter().map(|cv| cv.attribute_value_id));

        // Flatten the dependency graph into a single vec.
        let mut flattened_dependent_graph: Vec<&AttributeValueId> =
            request.payload.dependent_graph.keys().collect();
        flattened_dependent_graph.extend(request.payload.dependent_graph.values().flatten());

        // Send events according to every value in the dependency graph.
        let mut seen_code_generation_components: HashSet<ComponentId> = HashSet::new();
        let mut need_to_check_confirmations = true;
        for dependent_value in flattened_dependent_graph {
            if code_generation_attribute_values.contains(dependent_value) {
                let attribute_value = AttributeValue::get_by_id(&ctx, dependent_value)
                    .await?
                    .ok_or(AttributeValueError::NotFound(
                        *dependent_value,
                        *ctx.visibility(),
                    ))?;
                let component_id = attribute_value.context.component_id();
                if component_id != ComponentId::NONE
                    && !seen_code_generation_components.contains(&component_id)
                {
                    trace!("publishing code generated for component ({component_id}), tenancy ({:?}) and visibility ({:?})", *ctx.tenancy(), *ctx.visibility());
                    Self::publish_immediately(
                        &ctx,
                        WsEvent::code_generated(&ctx, component_id).await?,
                    )
                    .await?;
                    seen_code_generation_components.insert(component_id);
                }
            }

            // Only publish the confirmations event once.
            if need_to_check_confirmations
                && confirmation_attribute_values.contains(dependent_value)
            {
                trace!(
                    "publishing confirmations updated for tenancy ({:?}) and visibility ({:?})",
                    *ctx.tenancy(),
                    *ctx.visibility()
                );
                Self::publish_immediately(&ctx, WsEvent::confirmations_updated(&ctx).await?)
                    .await?;
                need_to_check_confirmations = false;
            }
        }

        Ok(())
    }

    /// Publish a [`WsEvent`](crate::WsEvent) immediately.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if the [`event`](crate::WsEvent) could not be published or the payload
    /// could not be serialized.
    ///
    /// # Notes
    ///
    /// This should only be done unless the caller is _certain_ that the [`event`](crate::WsEvent)
    /// should be published immediately. If unsure, use
    /// [`WsEvent::publish`](crate::WsEvent::publish_on_commit).
    ///
    /// This method requires an owned [`WsEvent`](crate::WsEvent), despite it not needing to,
    //  because [`events`](crate::WsEvent) should likely not be reused.
    async fn publish_immediately(ctx: &DalContext, ws_event: WsEvent) -> StatusReceiverResult<()> {
        let subject = format!("si.workspace_pk.{}.event", ws_event.workspace_pk());
        let msg_bytes = serde_json::to_vec(&ws_event)?;
        ctx.nats_conn().publish(subject, msg_bytes).await?;
        Ok(())
    }
}

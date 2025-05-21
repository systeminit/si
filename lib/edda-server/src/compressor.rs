use std::{
    collections::HashMap,
    task::Poll,
};

use bytes::Bytes;
use dal::{
    ChangeSetId,
    Component,
    DalContext,
    DalContextBuilder,
    TransactionsError,
    WorkspacePk,
    WorkspaceSnapshotAddress,
    workspace_snapshot::graph::detector::Update,
};
use edda_core::api_types::{
    RequestId,
    compressor_rebuild_request::{
        CompressorRebuildRequest,
        CompressorRebuildRequestVCurrent,
    },
    compressor_update_request::{
        CompressorUpdateRequest,
        CompressorUpdateRequestV1,
        CompressorUpdateRequestVCurrent,
    },
    rebuild_request,
};
use futures::{
    FutureExt,
    StreamExt,
};
use naxum::{
    MessageHead,
    extract::{
        FromMessage,
        FromMessageRaw,
    },
};
use pin_project_lite::pin_project;
use si_data_nats::async_nats::jetstream::{
    self,
    Message,
    consumer::{
        self,
        StreamError,
        push,
    },
    stream::ConsumerError,
};
use si_events::workspace_snapshot::Change;
use si_id::EntityId;
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::extract::{
    ApiTypesNegotiate,
    EddaRequestKind,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CompressorError {
    #[error("error creating consumer: {0}")]
    ConsumerCreate(#[source] ConsumerError),
    #[error("error building dal context: {0}")]
    DalContextBuild(#[source] TransactionsError),
    #[error("invalid edda request kind: compressor rebuild")]
    InvalidEddaRequestKindCompressorRebuild,
    #[error("invalid edda request kind: compressor update")]
    InvalidEddaRequestKindCompressorUpdate,
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("error while subscribing for messages: {0}")]
    Subscribe(#[source] StreamError),
}

type Result<T> = std::result::Result<T, CompressorError>;

#[remain::sorted]
#[derive(Debug, Clone)]
enum State {
    Draining(CompressorUpdateRequestVCurrent),
    DrainingToRebuild(CompressorRebuildRequestVCurrent),
    Inactive,
}

pin_project! {
    /// A stream that wraps an inner stream with the ability to "compress" requests.
    pub struct Compressor {
        incoming: consumer::push::Ordered,
        state: State,
        ctx_builder: DalContextBuilder,
        change_set_id: ChangeSetId,
        workspace_id: WorkspacePk,
        ctx: DalContext,
    }
}

impl Compressor {
    pub async fn new(
        requests_stream: &jetstream::stream::Stream,
        consumer_config: push::OrderedConfig,
        ctx_builder: DalContextBuilder,
        change_set_id: ChangeSetId,
        workspace_id: WorkspacePk,
    ) -> Result<Self> {
        let incoming = requests_stream
            .create_consumer(consumer_config)
            .await
            .map_err(CompressorError::ConsumerCreate)?
            .messages()
            .await
            .map_err(CompressorError::Subscribe)?;

        let ctx = ctx_builder
            .build_for_change_set_as_system(workspace_id, change_set_id, None)
            .await
            .map_err(CompressorError::DalContextBuild)?;

        Ok(Self {
            incoming,
            state: State::Inactive,
            ctx_builder,
            change_set_id,
            workspace_id,
            ctx,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompressorRequest {
    CompressorRebuild(CompressorRebuildRequestVCurrent),
    CompressorUpdate(CompressorUpdateRequestVCurrent),
}

impl futures::Stream for Compressor {
    type Item = std::result::Result<Message, consumer::push::MessagesError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let project = self.project();

        // FIXME(nick): we need to handle timing or max size. What if we have been pending for a long time or have made a massive batch?
        // We should publish the compressed message at certain limits. We need to first see if there are too many or too few compression
        // sets happening first though.

        loop {
            match project.incoming.poll_next_unpin(cx) {
                Poll::Ready(Some(result)) => {
                    // First, get the message off the stream and validate it.
                    let message = match result {
                        Ok(message) => message,
                        Err(err) => {
                            error!(si.error.message = ?err, "error getting info for jetstream message");
                            continue;
                        }
                    };

                    // FIXME(nick): remove later. Only used to make compiler happy to get this into draft PR form.
                    let cloned_message = message.clone();

                    // Determine if the stream has been drained.
                    let drained = match message.info() {
                        Ok(info) => info.pending == 0,
                        Err(err) => {
                            error!(si.error.message = ?err, "error getting info for jetstream message");
                            continue;
                        }
                    };

                    // Negotiate the API type. This is for handling streams with  multiple versions
                    // of messages.
                    let negotiated = match ApiTypesNegotiate::from_message_raw(message.into()) {
                        Ok(negotiated) => negotiated,
                        Err(err) => {
                            error!(si.error.message = ?err, "rejection or error when negotiating api type");
                            continue;
                        }
                    };

                    // Finally, try to compress the request. We need an inner loop because we need
                    // to fetch change batches from layer db... which returns a future.
                    let ctx = (*project.ctx).clone();
                    'inner: loop {
                        let locked_state = (*project.state).clone();
                        match Box::pin(compress(&ctx, negotiated.0.clone(), locked_state))
                            .poll_unpin(cx)
                        {
                            // If everything is drained, let's return the compressed message.
                            // This is the end of the critical path!
                            Poll::Ready(Ok(state)) if drained => {
                                let new_message = match state {
                                    State::Draining(inner) => {
                                        CompressorRequest::CompressorUpdate(inner)
                                    }
                                    State::DrainingToRebuild(inner) => {
                                        CompressorRequest::CompressorRebuild(inner)
                                    }
                                    // This should be impossible since compressing should always
                                    // result in an active state.
                                    State::Inactive => {
                                        error!("this should be impossible");
                                        break 'inner;
                                    }
                                };

                                // Reset the state and send the compressed message to the naxum handler.
                                // This is the end of the critical path!
                                *project.state = State::Inactive;

                                // FIXME(nick): I spent an hour trying to make naxum generic. Basically "Message<R>" is used everywhere
                                // and "R" must implement "MessageHead". I don't think it is worth pursuing that path. Frankly, I am
                                // not certain naxum is even the right tool for this. We should probably just double ack every message
                                // here and switch to more primitive Rust stream management semantics.
                                // ```
                                // message.payload =
                                //     Bytes::from(serde_json::to_value(new_message)?);
                                // ```

                                // Return the compressed message. This is the critical path!
                                return Poll::Ready(Some(Ok(cloned_message)));
                            }
                            // Everything is not drained so let's update the state and LOOP again!
                            // This is the critical path!
                            Poll::Ready(Ok(state)) => {
                                // FIXME(nick): double ack the message since it has been compressed.
                                *project.state = state;
                                break 'inner;
                            }
                            Poll::Ready(Err(err)) => {
                                error!(si.error.message = ?err, "compress error");
                                break 'inner;
                            }
                            // We are waiting for the layer db future.
                            Poll::Pending => {}
                        }
                    }
                }
                // FIXME(nick): handle shutdown and publishing what we have compressed in flight on the way out.
                Poll::Ready(None) => todo!(),
                // FIXME(nick): handle publishing what we have compressed in flight before returning pending.
                Poll::Pending => todo!(),
            }
        }
    }
}

async fn compress(ctx: &DalContext, request: EddaRequestKind, state: State) -> Result<State> {
    match (request, state) {
        // We see an update, are in a draining state, and the request's "from" snapshot matches
        // the current "to" snapshot. This is the critical path!
        (
            EddaRequestKind::Update(update_request),
            State::Draining(CompressorUpdateRequestVCurrent {
                id,
                previous_ids,
                from_snapshot_address: _from_snapshot_address,
                to_snapshot_address,
                changes,
            }),
        ) if update_request.from_snapshot_address == to_snapshot_address => match ctx
            .layer_db()
            .change_batch()
            .read_wait_for_memory(&update_request.change_batch_address)
            .await?
        {
            // We have found a change batch for the update request. We will update our current
            // compressed payload accordingly. This is the critical path!
            Some(change_batch) => {
                let mut changes = changes;
                for change in change_batch.changes() {
                    changes.insert(change.entity_id, change.to_owned());
                }
                let from_snapshot_address = update_request.from_snapshot_address;
                let to_snapshot_address = update_request.to_snapshot_address;
                let mut previous_ids = previous_ids;
                previous_ids.push(id);
                let id = update_request.id;
                Ok(State::Draining(CompressorUpdateRequestVCurrent {
                    id,
                    previous_ids,
                    from_snapshot_address,
                    to_snapshot_address,
                    changes,
                }))
            }
            // We have not found the change batch and are unable to determine what changes need to
            // be done. Let's go into rebuild mode.
            None => {
                let mut previous_ids = previous_ids;
                previous_ids.push(id);
                let id = update_request.id;
                Ok(State::DrainingToRebuild(CompressorRebuildRequestVCurrent {
                    id,
                    previous_ids,
                }))
            }
        },
        // We see an update, are in a draining state, BUT the request's "from" snapshot DOES NOT
        // match the current "to" snapshot. We are dealing with a non-contiguous chain and must go
        // into rebuild mode.
        (
            EddaRequestKind::Update(update_request),
            State::Draining(CompressorUpdateRequestVCurrent {
                id,
                previous_ids,
                from_snapshot_address: _from_snapshot_address,
                to_snapshot_address: _to_snapshot_address,
                changes: _changes,
            }),
        ) => {
            let mut previous_ids = previous_ids;
            previous_ids.push(id);
            let id = update_request.id;
            Ok(State::DrainingToRebuild(CompressorRebuildRequestVCurrent {
                id,
                previous_ids,
            }))
        }
        // We see an update and are in a draining-to-rebuild state. Let's update its fields.
        (
            EddaRequestKind::Update(update_request),
            State::DrainingToRebuild(CompressorRebuildRequestVCurrent { id, previous_ids }),
        ) => {
            let mut previous_ids = previous_ids;
            previous_ids.push(id);
            let id = update_request.id;
            Ok(State::DrainingToRebuild(CompressorRebuildRequestVCurrent {
                id,
                previous_ids,
            }))
        }
        // We see an update and are in an inactive state. We need to kick off the draining process.
        // This is start of the critical path!
        (EddaRequestKind::Update(update_request), State::Inactive) => match ctx
            .layer_db()
            .change_batch()
            .read_wait_for_memory(&update_request.change_batch_address)
            .await?
        {
            // We found a change batch for the update request. Let's initialize the update request.
            // This is the start of the critical path!
            Some(change_batch) => {
                let mut changes = HashMap::new();
                for change in change_batch.changes() {
                    changes.insert(change.entity_id, change.to_owned());
                }
                Ok(State::Draining(CompressorUpdateRequestVCurrent {
                    id: update_request.id,
                    previous_ids: Vec::new(),
                    from_snapshot_address: update_request.from_snapshot_address,
                    to_snapshot_address: update_request.to_snapshot_address,
                    changes,
                }))
            }
            // We have not found the change batch and are unable to determine what changes need to
            // be done. Let's initialize into rebuild mode.
            None => Ok(State::DrainingToRebuild(CompressorRebuildRequestVCurrent {
                id: update_request.id,
                previous_ids: Vec::new(),
            })),
        },
        // We see a rebuild request and we are currently draining. We need to swap to
        // draining-to-rebuild mode.
        (
            EddaRequestKind::Rebuild(rebuild_request),
            State::Draining(CompressorUpdateRequestVCurrent {
                id,
                previous_ids,
                from_snapshot_address: _from_snapshot_address,
                to_snapshot_address: _to_snapshot_address,
                changes: _changes,
            }),
        ) => {
            let mut previous_ids = previous_ids;
            previous_ids.push(id);
            let id = rebuild_request.id;
            Ok(State::DrainingToRebuild(CompressorRebuildRequestVCurrent {
                id,
                previous_ids,
            }))
        }
        // We see a rebuild request and are in a draining-to-rebuild state. Let's update its fields.
        (
            EddaRequestKind::Rebuild(rebuild_request),
            State::DrainingToRebuild(CompressorRebuildRequestVCurrent { id, previous_ids }),
        ) => {
            let mut previous_ids = previous_ids;
            previous_ids.push(id);
            let id = rebuild_request.id;
            Ok(State::DrainingToRebuild(CompressorRebuildRequestVCurrent {
                id,
                previous_ids,
            }))
        }
        // We see a rebuild request and are in an inactive state. Let's initialize into rebuild mode.
        (EddaRequestKind::Rebuild(rebuild_request), State::Inactive) => {
            Ok(State::DrainingToRebuild(CompressorRebuildRequestVCurrent {
                id: rebuild_request.id,
                previous_ids: Vec::new(),
            }))
        }
        // We see a compressor update request. This should be impossible as it should only be created
        // in-memory.
        (EddaRequestKind::CompressorUpdate(_), _) => {
            // FIXME(nick): depending on how we handle the naxum streams, we may not need compressor requests to be under this type.
            Err(CompressorError::InvalidEddaRequestKindCompressorUpdate)
        }
        // We see a compressor build request. This should be impossible as it should only be created
        // in-memory.
        (EddaRequestKind::CompressorRebuild(_), _) => {
            // FIXME(nick): depending on how we handle the naxum streams, we may not need compressor requests to be under this type.
            Err(CompressorError::InvalidEddaRequestKindCompressorRebuild)
        }
    }
}

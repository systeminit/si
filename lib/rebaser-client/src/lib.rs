use anyhow::Result;
use futures::{future::BoxFuture, StreamExt as _};
use pending_events::{PendingEventsError, PendingEventsStream};
use rebaser_core::{
    api_types::HeaderMapParseMessageInfoError,
    api_types::{
        enqueue_updates_request::{EnqueueUpdatesRequest, EnqueueUpdatesRequestVCurrent},
        enqueue_updates_response::EnqueueUpdatesResponse,
    },
    api_types::{
        ApiVersionsWrapper, ApiWrapper, ContentInfo, DeserializeError, SerializeError, UpgradeError,
    },
    nats::{self, NATS_HEADER_REPLY_INBOX_NAME},
};
use si_data_nats::{
    async_nats::{self, jetstream::context::PublishError},
    header,
    jetstream::{self, Context},
    HeaderMap, Message, NatsClient, Subject,
};
use si_events::{
    rebase_batch_address::RebaseBatchAddress, ChangeSetId, EventSessionId, WorkspacePk,
};
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;

pub use rebaser_core::{api_types, api_types::RequestId};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("error creating jetstream stream: {0}")]
    CreateStream(#[source] async_nats::jetstream::context::CreateStreamError),
    #[error("pending events error: {0}")]
    PendingEvents(#[from] PendingEventsError),
    #[error("request publish error: {0}")]
    Publish(#[from] PublishError),
    #[error("error deserializing reply: {0}")]
    ReplyDeserialize(#[from] DeserializeError),
    #[error("error parsing reply headers: {0}")]
    ReplyHeadersParse(#[from] HeaderMapParseMessageInfoError),
    #[error("reply message is missing headers")]
    ReplyMissingHeaders,
    #[error("reply subscription closed before receiving reply message")]
    ReplySubscriptionClosed,
    #[error("reply message has unsupported content type")]
    ReplyUnsupportedContentType,
    #[error("reply message has unsupported message type")]
    ReplyUnsupportedMessageType,
    #[error("reply message has unsupported message version")]
    ReplyUnsupportedMessageVersion,
    #[error("error upgrading reply message: {0}")]
    ReplyUpgrade(#[from] UpgradeError),
    #[error("error serializing request: {0}")]
    Serialize(#[from] SerializeError),
    #[error("reply subscribe error: {0}")]
    Subscribe(#[source] si_data_nats::Error),
}

type Error = ClientError;

pub type RebaserClient = Client;

#[derive(Clone, Debug)]
pub struct Client {
    nats: NatsClient,
    context: Context,
}

impl Client {
    pub async fn new(nats: NatsClient) -> Result<Self> {
        let context = jetstream::new(nats.clone());

        // Ensure that the streams are already created
        let _ = nats::rebaser_tasks_jetstream_stream(&context)
            .await
            .map_err(Error::CreateStream)?;
        let _ = nats::rebaser_requests_jetstream_stream(&context)
            .await
            .map_err(Error::CreateStream)?;

        Ok(Self { nats, context })
    }

    /// Asynchronously enqueues graph updates for processing by a Rebaser & return a [`RequestId`].
    #[instrument(
        name = "rebaser_client.enqueue_updates",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = %change_set_id,
            si.workspace.id = %workspace_id,
        ),
    )]
    pub async fn enqueue_updates(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddress,
        event_session_id: EventSessionId,
    ) -> Result<RequestId> {
        self.call_async(
            workspace_id,
            change_set_id,
            updates_address,
            None,
            None,
            event_session_id,
        )
        .await
    }

    /// Asynchronously enqueues graph updates that originate from a Change Set & return a
    /// [`RequestId`].
    #[instrument(
        name = "rebaser_client.enqueue_updates_from_change_set",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = %change_set_id,
            si.workspace.id = %workspace_id,
        ),
    )]
    pub async fn enqueue_updates_from_change_set(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddress,
        from_change_set_id: ChangeSetId,
        event_session_id: EventSessionId,
    ) -> Result<RequestId> {
        self.call_async(
            workspace_id,
            change_set_id,
            updates_address,
            Some(from_change_set_id),
            None,
            event_session_id,
        )
        .await
    }

    /// Enqueues graph updates for processing by a Rebaser and return a [`Future`] that will await
    /// the Rebaser's response with status.
    #[instrument(
        name = "rebaser_client.enqueue_updates_with_reply",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = %change_set_id,
            si.workspace.id = %workspace_id,
        ),
    )]
    pub async fn enqueue_updates_with_reply(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddress,
        event_session_id: EventSessionId,
    ) -> Result<(
        RequestId,
        BoxFuture<'static, Result<EnqueueUpdatesResponse>>,
    )> {
        self.call_with_reply(
            workspace_id,
            change_set_id,
            updates_address,
            None,
            event_session_id,
        )
        .await
    }

    /// Enqueues graph updates that originate from a Change Set and return a [`Future`] that will
    /// await the Rebaser's response with status.
    #[instrument(
        name = "rebaser_client.enqueue_updates_from_change_set_with_reply",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = %change_set_id,
            si.workspace.id = %workspace_id,
        ),
    )]
    pub async fn enqueue_updates_from_change_set_with_reply(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddress,
        from_change_set_id: ChangeSetId,
        event_session_id: EventSessionId,
    ) -> Result<(
        RequestId,
        BoxFuture<'static, Result<EnqueueUpdatesResponse>>,
    )> {
        self.call_with_reply(
            workspace_id,
            change_set_id,
            updates_address,
            Some(from_change_set_id),
            event_session_id,
        )
        .await
    }

    async fn call_async(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddress,
        from_change_set_id: Option<ChangeSetId>,
        maybe_reply_inbox: Option<&Subject>,
        event_session_id: EventSessionId,
    ) -> Result<RequestId> {
        let id = RequestId::new();

        let pending_events_stream =
            PendingEventsStream::get_or_create(self.context.to_owned()).await?;
        pending_events_stream
            .publish_audit_log_final_message(workspace_id, change_set_id, event_session_id)
            .await?;

        let request = EnqueueUpdatesRequest::new_current(EnqueueUpdatesRequestVCurrent {
            id,
            workspace_id,
            change_set_id,
            updates_address,
            from_change_set_id,
            event_session_id: Some(event_session_id),
        });

        // Cut down on the amount of `String` allocations dealing with ids
        let mut wid_buf = [0; WorkspacePk::ID_LEN];
        let mut csid_buf = [0; ChangeSetId::ID_LEN];

        let requests_subject = nats::subject::enqueue_updates_for_change_set(
            self.context.metadata().subject_prefix(),
            workspace_id.array_to_str(&mut wid_buf),
            change_set_id.array_to_str(&mut csid_buf),
        );

        let info = ContentInfo::from(&request);

        let mut headers = HeaderMap::new();
        propagation::inject_headers(&mut headers);
        info.inject_into_headers(&mut headers);
        headers.insert(header::NATS_MESSAGE_ID, id.to_string());
        if let Some(reply_inbox) = maybe_reply_inbox {
            headers.insert(NATS_HEADER_REPLY_INBOX_NAME, reply_inbox.as_str());
        }

        self.context
            .publish_with_headers(requests_subject, headers, request.to_vec()?.into())
            .await?
            .await?;

        let tasks_subject = nats::subject::process_task_for_change_set(
            self.context.metadata().subject_prefix(),
            workspace_id.array_to_str(&mut wid_buf),
            change_set_id.array_to_str(&mut csid_buf),
        );

        // There is one more optional future here which is confirmation from the NATS server that
        // our publish was acked. However, the task stream will drop new messages that are
        // duplicates and this returns an error on the "ack future". Instead, we'll keep this as
        // fire and forget.
        self.context.publish(tasks_subject, vec![].into()).await?;

        Ok(id)
    }

    async fn call_with_reply(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddress,
        from_change_set_id: Option<ChangeSetId>,
        event_session_id: EventSessionId,
    ) -> Result<(
        RequestId,
        BoxFuture<'static, Result<EnqueueUpdatesResponse>>,
    )> {
        let reply_inbox: Subject = self.nats.new_inbox().into();

        trace!(
            messaging.destination = &reply_inbox.as_str(),
            "subscribing for reply message"
        );
        let mut subscription = self
            .nats
            .subscribe(reply_inbox.clone())
            .await
            .map_err(Error::Subscribe)?;
        subscription
            .unsubscribe_after(1)
            .await
            .map_err(Error::Subscribe)?;

        let id = self
            .call_async(
                workspace_id,
                change_set_id,
                updates_address,
                from_change_set_id,
                Some(&reply_inbox),
                event_session_id,
            )
            .await?;

        let fut = Box::pin(async move {
            let reply = subscription
                .next()
                .await
                .ok_or(Error::ReplySubscriptionClosed)?;

            propagation::associate_current_span_from_headers(reply.headers());

            response_from_reply(reply)
        });

        Ok((id, fut))
    }
}

fn response_from_reply<T>(message: Message) -> Result<T>
where
    T: ApiWrapper,
{
    let headers = message.headers().ok_or(Error::ReplyMissingHeaders)?;
    let info = ContentInfo::try_from(headers)?;
    if !T::is_content_type_supported(info.content_type.as_str()) {
        return Err(Error::ReplyUnsupportedContentType.into());
    }
    if !T::is_message_type_supported(info.message_type.as_str()) {
        return Err(Error::ReplyUnsupportedMessageType.into());
    }
    if !T::is_message_version_supported(info.message_version.as_u64()) {
        return Err(Error::ReplyUnsupportedMessageVersion.into());
    }

    let deserialized_version = T::from_slice(info.content_type.as_str(), message.payload())?;
    let current_version = deserialized_version.into_current_version()?;

    Ok(current_version)
}

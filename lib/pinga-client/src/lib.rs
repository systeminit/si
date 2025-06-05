use std::result;

use futures::{
    StreamExt as _,
    future::BoxFuture,
};
use nats_std::headers;
pub use pinga_core::{
    api_types,
    api_types::RequestId,
};
use pinga_core::{
    api_types::{
        ApiVersionsWrapper,
        ApiWrapper,
        ContentInfo,
        DeserializeError,
        HeaderMapParseMessageInfoError,
        SerializeError,
        UpgradeError,
        job_execution_request::{
            JobArgsVCurrent,
            JobExecutionRequest,
            JobExecutionRequestVCurrent,
        },
        job_execution_response::JobExecutionResponse,
    },
    nats,
};
use si_data_nats::{
    HeaderMap,
    Message,
    NatsClient,
    Subject,
    async_nats::{
        self,
        jetstream::context::PublishError,
    },
    header,
    jetstream::{
        self,
        Context,
    },
};
use si_events::{
    ActionId,
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    ManagementPrototypeId,
    ViewId,
    WorkspacePk,
    ulid::CoreUlid,
};
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("error creating jetstream stream: {0}")]
    CreateStream(#[source] async_nats::jetstream::context::CreateStreamError),
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

type Result<T> = result::Result<T, ClientError>;

pub type PingaClient = Client;

/// A client which can submit job execution requests to Pinga services.
#[derive(Clone, Debug)]
pub struct Client {
    nats: NatsClient,
    context: Context,
}

impl Client {
    /// Builds and returns a new [`Client`].
    pub async fn new(nats: NatsClient) -> Result<Self> {
        let context = jetstream::new(nats.clone());

        // Ensure that the stream is already created
        let _ = nats::pinga_work_queue(&context)
            .await
            .map_err(Error::CreateStream)?;

        Ok(Self { nats, context })
    }

    /// Requests an action job execution and returns an awaitable response future.
    pub async fn await_action_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        action_id: ActionId,
        is_job_blocking: bool,
    ) -> Result<(RequestId, BoxFuture<'static, Result<JobExecutionResponse>>)> {
        self.call_with_reply(
            workspace_id,
            change_set_id,
            JobArgsVCurrent::Action { action_id },
            is_job_blocking,
        )
        .await
    }

    /// Requests a dependent values update job execution and returns an awaitable response future.
    pub async fn await_dependent_values_update_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        is_job_blocking: bool,
    ) -> Result<(RequestId, BoxFuture<'static, Result<JobExecutionResponse>>)> {
        self.call_with_reply(
            workspace_id,
            change_set_id,
            JobArgsVCurrent::DependentValuesUpdate,
            is_job_blocking,
        )
        .await
    }

    /// Requests a validation job execution and returns an awaitable response future.
    pub async fn await_validation_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        attribute_value_ids: Vec<AttributeValueId>,
        is_job_blocking: bool,
    ) -> Result<(RequestId, BoxFuture<'static, Result<JobExecutionResponse>>)> {
        self.call_with_reply(
            workspace_id,
            change_set_id,
            JobArgsVCurrent::Validation {
                attribute_value_ids,
            },
            is_job_blocking,
        )
        .await
    }

    /// Requests a management job execution and returns an awaitable response future.
    #[allow(clippy::too_many_arguments)]
    pub async fn await_management_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        component_id: ComponentId,
        prototype_id: ManagementPrototypeId,
        view_id: ViewId,
        request_ulid: Option<CoreUlid>,
        is_job_blocking: bool,
    ) -> Result<(RequestId, BoxFuture<'static, Result<JobExecutionResponse>>)> {
        self.call_with_reply(
            workspace_id,
            change_set_id,
            JobArgsVCurrent::ManagementFunc {
                component_id,
                prototype_id,
                view_id,
                request_ulid,
            },
            is_job_blocking,
        )
        .await
    }

    /// Requests an action job execution and doesnt't wait for a response.
    pub async fn dispatch_action_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        action_id: ActionId,
        is_job_blocking: bool,
    ) -> Result<RequestId> {
        self.call_async(
            workspace_id,
            change_set_id,
            JobArgsVCurrent::Action { action_id },
            is_job_blocking,
            None,
        )
        .await
    }

    /// Requests a dependent values update job execution and doesnt't wait for a response.
    pub async fn dispatch_dependent_values_update_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        is_job_blocking: bool,
    ) -> Result<RequestId> {
        self.call_async(
            workspace_id,
            change_set_id,
            JobArgsVCurrent::DependentValuesUpdate,
            is_job_blocking,
            None,
        )
        .await
    }

    /// Requests a validation job execution and doesnt't wait for a response.
    pub async fn dispatch_validation_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        attribute_value_ids: Vec<AttributeValueId>,
        is_job_blocking: bool,
    ) -> Result<RequestId> {
        self.call_async(
            workspace_id,
            change_set_id,
            JobArgsVCurrent::Validation {
                attribute_value_ids,
            },
            is_job_blocking,
            None,
        )
        .await
    }

    /// Requests a management job execution and doesnt't wait for a response.
    #[allow(clippy::too_many_arguments)]
    pub async fn dispatch_management_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        component_id: ComponentId,
        prototype_id: ManagementPrototypeId,
        view_id: ViewId,
        request_ulid: Option<CoreUlid>,
        is_job_blocking: bool,
    ) -> Result<RequestId> {
        self.call_async(
            workspace_id,
            change_set_id,
            JobArgsVCurrent::ManagementFunc {
                component_id,
                prototype_id,
                view_id,
                request_ulid,
            },
            is_job_blocking,
            None,
        )
        .await
    }

    async fn call_async(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        args: JobArgsVCurrent,
        is_job_blocking: bool,
        maybe_reply_inbox: Option<&Subject>,
    ) -> Result<RequestId> {
        let id = RequestId::new();

        let kind: &'static str = (&args).into();

        let request = JobExecutionRequest::new_current(JobExecutionRequestVCurrent {
            id,
            workspace_id,
            change_set_id,
            args,
            is_job_blocking,
        });

        // Cut down on the amount of `String` allocations dealing with ids
        let mut wid_buf = [0; WorkspacePk::ID_LEN];
        let mut csid_buf = [0; ChangeSetId::ID_LEN];

        let requests_subject = nats::subject::pinga_job(
            self.context.metadata().subject_prefix(),
            workspace_id.array_to_str(&mut wid_buf),
            change_set_id.array_to_str(&mut csid_buf),
            kind,
        );

        let info = ContentInfo::from(&request);

        let mut headers = HeaderMap::new();
        propagation::inject_headers(&mut headers);
        info.inject_into_headers(&mut headers);
        headers.insert(header::NATS_MESSAGE_ID, id.to_string());
        headers::insert_maybe_reply_inbox(&mut headers, maybe_reply_inbox);

        self.context
            .publish_with_headers(requests_subject, headers, request.to_vec()?.into())
            .await?
            .await?;

        Ok(id)
    }

    async fn call_with_reply(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        args: JobArgsVCurrent,
        is_job_blocking: bool,
    ) -> Result<(RequestId, BoxFuture<'static, Result<JobExecutionResponse>>)> {
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
                args,
                is_job_blocking,
                Some(&reply_inbox),
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
        return Err(Error::ReplyUnsupportedContentType);
    }
    if !T::is_message_type_supported(info.message_type.as_str()) {
        return Err(Error::ReplyUnsupportedMessageType);
    }
    if !T::is_message_version_supported(info.message_version.as_u64()) {
        return Err(Error::ReplyUnsupportedMessageVersion);
    }

    let deserialized_version = T::from_slice(info.content_type.as_str(), message.payload())?;
    let current_version = deserialized_version.into_current_version()?;

    Ok(current_version)
}

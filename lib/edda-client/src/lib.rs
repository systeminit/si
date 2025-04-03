use std::result;

use bytes::Bytes;
use edda_core::{
    api_types::{
        rebuild_request::{RebuildRequest, RebuildRequestVCurrent},
        update_request::{UpdateRequest, UpdateRequestVCurrent},
        ApiWrapper, ContentInfo, DeserializeError, HeaderMapParseMessageInfoError, RequestId,
        SerializeError, UpgradeError,
    },
    nats,
};
use si_data_nats::{
    async_nats::{self, jetstream::context::PublishError},
    header,
    jetstream::{self, Context},
    HeaderMap, NatsClient,
};
use si_events::{
    change_batch::ChangeBatchAddress, ChangeSetId, WorkspacePk, WorkspaceSnapshotAddress,
};
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

pub type EddaClient = Client;

#[derive(Clone, Debug)]
pub struct Client {
    context: Context,
}

impl Client {
    pub async fn new(nats: NatsClient) -> Result<Self> {
        let context = jetstream::new(nats);

        // Ensure that the streams are already created
        let _ = nats::edda_tasks_jetstream_stream(&context)
            .await
            .map_err(Error::CreateStream)?;
        let _ = nats::edda_requests_jetstream_stream(&context)
            .await
            .map_err(Error::CreateStream)?;

        Ok(Self { context })
    }

    /// Asynchronously request an index update from a workspace past snapshot to the current
    /// snapshot & return a [`RequestId`].
    pub async fn update_from_workspace_snapshot(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        from_snapshot_address: WorkspaceSnapshotAddress,
        to_snapshot_address: WorkspaceSnapshotAddress,
        change_batch_address: ChangeBatchAddress,
    ) -> Result<RequestId> {
        let id = RequestId::new();
        let request = UpdateRequest::new_current(UpdateRequestVCurrent {
            id,
            from_snapshot_address,
            to_snapshot_address,
            change_batch_address,
        });
        let info = ContentInfo::from(&request);

        self.publish_inner(
            workspace_id,
            change_set_id,
            id,
            request.to_vec()?.into(),
            info,
        )
        .await
    }

    pub async fn rebuild_for_change_set(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> Result<RequestId> {
        let id = RequestId::new();
        let request = RebuildRequest::new_current(RebuildRequestVCurrent { id });
        let info = ContentInfo::from(&request);

        self.publish_inner(
            workspace_id,
            change_set_id,
            id,
            request.to_vec()?.into(),
            info,
        )
        .await
    }

    async fn publish_inner(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        id: RequestId,
        payload: Bytes,
        info: ContentInfo<'_>,
    ) -> Result<RequestId> {
        // Cut down on the amount of `String` allocations dealing with ids
        let mut wid_buf = [0; WorkspacePk::ID_LEN];
        let mut csid_buf = [0; ChangeSetId::ID_LEN];

        let requests_subject = nats::subject::request_for_change_set(
            self.context.metadata().subject_prefix(),
            workspace_id.array_to_str(&mut wid_buf),
            change_set_id.array_to_str(&mut csid_buf),
        );

        let mut headers = HeaderMap::new();
        propagation::inject_headers(&mut headers);
        info.inject_into_headers(&mut headers);
        headers.insert(header::NATS_MESSAGE_ID, id.to_string());

        self.context
            .publish_with_headers(requests_subject, headers, payload)
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

    // TODO(fnichol): add method to be called from SDF where we get either a NATS k/v watch or some
    // `impl Future` that resolves when the index is updated (still using a k/v watch under the
    // hood)
}

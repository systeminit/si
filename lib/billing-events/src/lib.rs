use serde::{Deserialize, Serialize};
use serde_json::Error;
use si_data_nats::{
    async_nats::jetstream::{
        context::{CreateStreamError, PublishError},
        stream::{Config, DiscardPolicy, RetentionPolicy, Stream},
    },
    jetstream,
};
use si_events::{ChangeSetId, UserPk, WorkspacePk, WorkspaceSnapshotAddress};
use thiserror::Error;

const STREAM_NAME: &str = "BILLING_EVENTS";
const WORKSPACE_UPDATE_SUBJECT: &str = "billing.workspace_update";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum BillingEventsError {
    #[error("create stream error: {0}")]
    CreateStream(#[from] CreateStreamError),
    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] Error),
}

pub type BillingEventsResult<T> = Result<T, BillingEventsError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingWorkspaceChangeEvent {
    /// The workspace of this change
    pub workspace: WorkspacePk,

    /// The resource count of the workspace
    pub resource_count: u64,

    /// Description of the change that caused the event
    pub change_description: String,

    // TODO(nick,jkeiser): ensure that "ChangeSetStatus" can move to si-events-rs within needing to pull in si-data-pg
    // for its "ToSql" implementation.
    /// The status of the workspace
    pub status: String,
    /// The specific snapshot with this resource count
    pub workspace_snapshot_address: WorkspaceSnapshotAddress,
    /// The change set id of the count (should only ever be main, but this is for reconciliation)
    pub change_set_id: ChangeSetId,
    /// The user who requested the update (if any)
    pub merge_requested_by_user_id: Option<UserPk>,
}

#[derive(Debug, Clone)]
pub struct BillingEventsWorkQueue {
    context: jetstream::Context,
}

impl BillingEventsWorkQueue {
    /// Create a new instance of the billing events work queue.
    ///
    /// Ensures the stream is created.
    pub async fn get_or_create(context: jetstream::Context) -> BillingEventsResult<Self> {
        // Ensure the stream is created before we start publishing to it
        let result = Self { context };
        result.stream().await?;
        Ok(result)
    }

    pub fn name(&self) -> &str {
        STREAM_NAME
    }

    /// Publish a workspace update.
    pub async fn publish_workspace_update(
        &self,
        workspace_id: &str,
        message: &impl Serialize,
    ) -> BillingEventsResult<()> {
        self.publish_message(WORKSPACE_UPDATE_SUBJECT, workspace_id, message)
            .await
    }

    /// Get the events stream.
    pub async fn stream(&self) -> BillingEventsResult<Stream> {
        let config = Config {
            name: self.prefixed_stream_name(STREAM_NAME),
            description: Some("Billing actions work queue of events".to_string()),
            subjects: vec![self.prefixed_subject(WORKSPACE_UPDATE_SUBJECT, ">")],
            retention: RetentionPolicy::WorkQueue,
            allow_direct: true,
            // Enable this to apply backpressure (stop allowing changes until we can process
            // billing events that have already happened).
            // discard_new_per_subject: true
            discard: DiscardPolicy::New,
            // This prevents deletion entirely, only allowing messages to disappear when ACKed
            // deny_delete: true,
            // deny_purge: true,
            ..Default::default()
        };
        Ok(self.context.get_or_create_stream(config).await?)
    }

    /// Provides the [`WORKSPACE_UPDATE_SUBJECT`] with an appropriate prefix and suffix.
    pub fn workspace_update_subject(&self, suffix: &str) -> String {
        self.prefixed_subject(WORKSPACE_UPDATE_SUBJECT, suffix)
    }

    async fn publish_message(
        &self,
        subject: &str,
        parameters: &str,
        message: &impl Serialize,
    ) -> BillingEventsResult<()> {
        let subject = self.prefixed_subject(subject, parameters);
        let ack = self
            .context
            .publish(subject, serde_json::to_vec(message)?.into())
            .await?;
        ack.await?;
        Ok(())
    }

    fn prefixed_stream_name(&self, stream_name: &str) -> String {
        match self.context.metadata().subject_prefix() {
            Some(prefix) => format!("{prefix}_{stream_name}"),
            None => stream_name.to_owned(),
        }
    }

    fn prefixed_subject(&self, subject: &str, suffix: &str) -> String {
        match self.context.metadata().subject_prefix() {
            Some(prefix) => format!("{prefix}.{subject}.{suffix}"),
            None => format!("{subject}.{suffix}"),
        }
    }
}

use serde::{Deserialize, Serialize};
use si_events::rebase_batch_address::RebaseBatchAddress;
use strum::EnumDiscriminants;

use telemetry::prelude::*;
use telemetry::tracing::instrument;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::Instant;
use tokio_stream::wrappers::BroadcastStream;
use ulid::Ulid;

use super::{Activity, ActivityId, ActivityPayloadDiscriminants, ActivityRebaseRequest};
use crate::activity_client::ActivityClient;
use crate::{error::LayerDbResult, event::LayeredEventMetadata};

/// The message that the server receives to perform a rebase.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RebaseRequest {
    /// Corresponds to the change set whose pointer is to be updated.
    pub to_rebase_change_set_id: Ulid,
    /// Corresponds to the workspace snapshot that will be the "onto" workspace snapshot when
    /// rebasing the "to rebase" workspace snapshot.
    pub rebase_batch_address: RebaseBatchAddress,
    /// If the rebase batch comes from another change set (i.e. the default change set), then this
    /// field is set.
    pub from_change_set_id: Option<Ulid>,
}

impl RebaseRequest {
    pub fn new(
        to_rebase_change_set_id: Ulid,
        rebase_batch_address: RebaseBatchAddress,
        from_change_set_id: Option<Ulid>,
    ) -> RebaseRequest {
        RebaseRequest {
            to_rebase_change_set_id,
            rebase_batch_address,
            from_change_set_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RebaseFinished {
    status: RebaseStatus,
    to_rebase_change_set_id: Ulid,
    rebase_batch_address: RebaseBatchAddress,
}

impl RebaseFinished {
    pub fn new(
        status: RebaseStatus,
        to_rebase_change_set_id: Ulid,
        rebase_batch_address: RebaseBatchAddress,
    ) -> RebaseFinished {
        RebaseFinished {
            status,
            to_rebase_change_set_id,
            rebase_batch_address,
        }
    }

    pub fn status(&self) -> &RebaseStatus {
        &self.status
    }

    pub fn to_rebase_change_set_id(&self) -> &Ulid {
        &self.to_rebase_change_set_id
    }

    pub fn rebase_batch_address(&self) -> &RebaseBatchAddress {
        &self.rebase_batch_address
    }
}

// NOTE: We're basically smashing the data in here, and we really do have to figure out what we
// actually want when things work / or don't work.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize))]
pub enum RebaseStatus {
    /// Processing the request and performing updates were both successful. Additionally, no conflicts were found.
    Success {
        /// The serialized updates performed when rebasing.
        updates_performed: RebaseBatchAddress,
    },
    /// Error encountered when processing the request.
    Error {
        /// The error message.
        message: String,
    },
}

#[derive(Debug)]
pub struct ActivityRebase<'a> {
    activity_base: &'a ActivityClient,
}

impl<'a> ActivityRebase<'a> {
    pub fn new(activity_base: &'a ActivityClient) -> Self {
        Self { activity_base }
    }

    #[instrument(name = "activity::rebase::rebase", level = "info")]
    pub async fn rebase(
        &self,
        to_rebase_change_set_id: Ulid,
        rebase_batch_address: RebaseBatchAddress,
        metadata: LayeredEventMetadata,
    ) -> LayerDbResult<Activity> {
        let payload = RebaseRequest::new(to_rebase_change_set_id, rebase_batch_address, None);
        let activity = Activity::rebase(payload, metadata);
        self.activity_base.publish(&activity).await?;
        Ok(activity)
    }

    #[instrument(name = "activity::rebase::rebase", level = "info")]
    pub async fn rebase_from_change_set(
        &self,
        to_rebase_change_set_id: Ulid,
        rebase_batch_address: RebaseBatchAddress,
        from_change_set_id: Ulid,
        metadata: LayeredEventMetadata,
    ) -> LayerDbResult<Activity> {
        let payload = RebaseRequest::new(
            to_rebase_change_set_id,
            rebase_batch_address,
            Some(from_change_set_id),
        );
        let activity = Activity::rebase(payload, metadata);
        self.activity_base.publish(&activity).await?;
        Ok(activity)
    }

    #[instrument(name = "activity::rebase::rebase_and_wait", level = "info", skip(self))]
    pub async fn rebase_and_wait(
        &self,
        to_rebase_change_set_id: Ulid,
        rebase_batch_address: RebaseBatchAddress,
        metadata: LayeredEventMetadata,
    ) -> LayerDbResult<Activity> {
        let payload = RebaseRequest::new(to_rebase_change_set_id, rebase_batch_address, None);
        let activity = Activity::rebase(payload, metadata);
        // println!("trigger: sending rebase and waiting for response");
        debug!(?activity, "sending rebase and waiting for response");

        // Why is this in two? We want to start listening before the publish call, to ensure we
        // aren't racing with any listening service.
        let start = Instant::now();
        let rx = self.rebase_finished_activity_stream().await?;
        let join_handle =
            tokio::spawn(ActivityClient::wait_for_parent_activity_id(rx, activity.id));
        self.activity_base.publish(&activity).await?;
        let rebase_finished_activity = join_handle.await??;
        debug!(?rebase_finished_activity, elapsed = ?start.elapsed(), "received rebase finished");
        // println!("trigger: done rebase");

        Ok(rebase_finished_activity)
    }

    /// Returns an `impl Stream` of [`Activity`] items which are of interest to a Rebaser.
    pub async fn rebaser_activity_stream(&self) -> LayerDbResult<BroadcastStream<Activity>> {
        self.activity_base
            .subscribe(Some(ActivityPayloadDiscriminants::RebaseRequest))
            .await
    }

    pub async fn rebase_finished_activity_stream(
        &self,
    ) -> LayerDbResult<BroadcastStream<Activity>> {
        self.activity_base
            .subscribe(Some(ActivityPayloadDiscriminants::RebaseFinished))
            .await
    }

    #[instrument(name = "activity::rebase::rebase_finished", level = "info", skip(self))]
    pub async fn finished(
        &self,
        status: RebaseStatus,
        to_rebase_change_set_id: Ulid,
        rebase_batch_address: RebaseBatchAddress,
        metadata: LayeredEventMetadata,
        parent_activity_id: ActivityId,
    ) -> LayerDbResult<Activity> {
        let payload = RebaseFinished::new(status, to_rebase_change_set_id, rebase_batch_address);
        let activity = Activity::rebase_finished(payload, metadata, parent_activity_id);
        self.activity_base.publish(&activity).await?;
        Ok(activity)
    }

    pub async fn subscribe_work_queue(
        &self,
    ) -> LayerDbResult<UnboundedReceiver<ActivityRebaseRequest>> {
        self.activity_base.rebaser_request_work_queue().await
    }
}

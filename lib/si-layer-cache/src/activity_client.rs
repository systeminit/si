use std::{
    sync::Arc,
    time::Duration,
};

use si_data_nats::{
    NatsClient,
    jetstream::Context,
};
use telemetry::prelude::*;
use tokio::pin;
use tokio_stream::{
    StreamExt,
    wrappers::BroadcastStream,
};
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use crate::{
    activities::{
        Activity,
        ActivityId,
        ActivityMultiplexer,
        ActivityPayloadDiscriminants,
        ActivityPublisher,
        test::ActivityIntegrationTest,
    },
    error::{
        LayerDbError,
        LayerDbResult,
    },
};

const PARENT_ACTIVITY_WAIT_TIMEOUT: u64 = 60;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ActivityClient {
    instance_id: Ulid,
    context: Context,
    subject_prefix: Option<Arc<str>>,
    activity_publisher: ActivityPublisher,
    activity_multiplexer: ActivityMultiplexer,
    shutdown_token: CancellationToken,
}

impl ActivityClient {
    pub fn new(
        instance_id: Ulid,
        nats_client: NatsClient,
        shutdown_token: CancellationToken,
    ) -> ActivityClient {
        let subject_prefix = nats_client.metadata().subject_prefix().map(|s| s.into());
        let context = si_data_nats::jetstream::new(nats_client);

        let activity_publisher = ActivityPublisher::new(context.clone(), subject_prefix.clone());
        let activity_multiplexer = ActivityMultiplexer::new(
            instance_id,
            context.clone(),
            subject_prefix.clone(),
            shutdown_token.clone(),
        );

        ActivityClient {
            activity_publisher,
            activity_multiplexer,
            instance_id,
            context,
            subject_prefix,
            shutdown_token,
        }
    }

    pub fn activity_publisher(&self) -> &ActivityPublisher {
        &self.activity_publisher
    }

    pub fn activity_multiplexer(&self) -> &ActivityMultiplexer {
        &self.activity_multiplexer
    }

    // Publish an activity
    #[instrument(name = "activity_base::publish", level = "trace")]
    pub async fn publish(&self, activity: &Activity) -> LayerDbResult<()> {
        self.activity_publisher.publish(activity).await
    }

    // Subscribe to all activities, or provide an optional array of activity kinds
    // to subscribe to.
    pub async fn subscribe(
        &self,
        to_receive: impl IntoIterator<Item = ActivityPayloadDiscriminants>,
    ) -> LayerDbResult<BroadcastStream<Activity>> {
        Ok(BroadcastStream::new(
            self.activity_multiplexer
                .subscribe(Some(to_receive))
                .await?,
        ))
    }

    pub async fn subscribe_all(&self) -> LayerDbResult<BroadcastStream<Activity>> {
        Ok(BroadcastStream::new(
            self.activity_multiplexer
                .subscribe(None::<std::vec::IntoIter<_>>)
                .await?,
        ))
    }

    pub async fn wait_for_parent_activity_id(
        stream: BroadcastStream<Activity>,
        wait_for_parent_activity_id: ActivityId,
    ) -> LayerDbResult<Activity> {
        let filter_stream = stream.filter(move |activity_result| {
            if let Ok(activity) = activity_result {
                if let Some(parent_activity_id) = activity.parent_activity_id {
                    parent_activity_id == wait_for_parent_activity_id
                } else {
                    false
                }
            } else {
                false
            }
        });
        let timeout_stream =
            filter_stream.timeout(Duration::from_secs(PARENT_ACTIVITY_WAIT_TIMEOUT));
        pin!(timeout_stream);
        if let Some(activity_result_or_timeout) = timeout_stream.next().await {
            match activity_result_or_timeout {
                Ok(activity_result) => match activity_result {
                    Ok(activity) => return Ok(activity),
                    Err(_) => {
                        return Err(LayerDbError::ActivityWaitLagged(
                            wait_for_parent_activity_id,
                        ));
                    }
                },
                Err(elapsed) => {
                    return Err(LayerDbError::ActivityWaitTimeout(
                        wait_for_parent_activity_id,
                        elapsed,
                    ));
                }
            }
        }
        Err(LayerDbError::ActivityWaitClosed(
            wait_for_parent_activity_id,
        ))
    }

    pub fn test(&self) -> ActivityIntegrationTest {
        ActivityIntegrationTest::new(self)
    }
}

use std::time::Duration;

use si_data_nats::NatsClient;
use telemetry::prelude::*;
use tokio::pin;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

use crate::activities::rebase::ActivityRebase;
use crate::activities::test::ActivityIntegrationTest;
use crate::activities::{
    Activity, ActivityId, ActivityMultiplexer, ActivityPayloadDiscriminants, ActivityPublisher,
};
use crate::error::{LayerDbError, LayerDbResult};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ActivityClient {
    instance_id: Ulid,
    nats_client: NatsClient,
    activity_publisher: ActivityPublisher,
    activity_multiplexer: ActivityMultiplexer,
}

impl ActivityClient {
    pub fn new(
        instance_id: Ulid,
        nats_client: NatsClient,
        shutdown_token: CancellationToken,
    ) -> ActivityClient {
        let activity_publisher = ActivityPublisher::new(&nats_client);
        let activity_multiplexer =
            ActivityMultiplexer::new(instance_id, nats_client.clone(), shutdown_token);

        ActivityClient {
            activity_publisher,
            activity_multiplexer,
            instance_id,
            nats_client,
        }
    }

    pub fn nats_client(&self) -> &NatsClient {
        &self.nats_client
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
        let timeout_stream = filter_stream.timeout(Duration::from_secs(30));
        pin!(timeout_stream);
        if let Some(activity_result_or_timeout) = timeout_stream.next().await {
            match activity_result_or_timeout {
                Ok(activity_result) => match activity_result {
                    Ok(activity) => return Ok(activity),
                    Err(_) => {
                        return Err(LayerDbError::ActivityWaitLagged(
                            wait_for_parent_activity_id,
                        ))
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

    pub fn rebase(&self) -> ActivityRebase {
        ActivityRebase::new(self)
    }

    pub fn test(&self) -> ActivityIntegrationTest {
        ActivityIntegrationTest::new(self)
    }
}

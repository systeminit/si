use serde::{Deserialize, Serialize};

use crate::{activity_client::ActivityClient, error::LayerDbResult, event::LayeredEventMetadata};

use super::{Activity, ActivityId, ActivityPayload};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct IntegrationTest {
    pub name: String,
}

impl IntegrationTest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct IntegrationTestAlt {
    pub name: String,
}

impl IntegrationTestAlt {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug)]
pub struct ActivityIntegrationTest<'a> {
    activity_client: &'a ActivityClient,
}

impl<'a> ActivityIntegrationTest<'a> {
    pub fn new(activity_client: &'a ActivityClient) -> Self {
        Self { activity_client }
    }

    pub async fn integration_test(
        &self,
        name: impl Into<String>,
        metadata: LayeredEventMetadata,
        parent_activity_id: Option<ActivityId>,
    ) -> LayerDbResult<Activity> {
        let activity =
            Activity::integration_test(IntegrationTest::new(name), metadata, parent_activity_id);
        self.activity_client.publish(&activity).await?;
        Ok(activity)
    }

    pub async fn integration_test_alt(
        &self,
        name: impl Into<String>,
        metadata: LayeredEventMetadata,
        parent_activity_id: Option<ActivityId>,
    ) -> LayerDbResult<Activity> {
        let activity = Activity::integration_test_alt(
            IntegrationTestAlt::new(name),
            metadata,
            parent_activity_id,
        );
        self.activity_client.publish(&activity).await?;
        Ok(activity)
    }
}

impl Activity {
    pub fn integration_test(
        payload: IntegrationTest,
        metadata: LayeredEventMetadata,
        parent_activity_id: Option<ActivityId>,
    ) -> Activity {
        Activity::new(
            ActivityPayload::IntegrationTest(payload),
            metadata,
            parent_activity_id,
        )
    }

    pub fn integration_test_alt(
        payload: IntegrationTestAlt,
        metadata: LayeredEventMetadata,
        parent_activity_id: Option<ActivityId>,
    ) -> Activity {
        Activity::new(
            ActivityPayload::IntegrationTestAlt(payload),
            metadata,
            parent_activity_id,
        )
    }
}

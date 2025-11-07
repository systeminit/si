//! This module contains the ability to work with "resources" for [`Components`](crate::Component).

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use veritech_client::ActionRunResultSuccess;
pub use veritech_client::ResourceStatus;

use crate::{
    Component,
    ComponentId,
    DalContext,
    component::ComponentResult,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ResourceData {
    pub status: ResourceStatus,
    pub payload: Option<serde_json::Value>,
    pub last_synced: DateTime<Utc>,
}

impl ResourceData {
    pub fn new(status: ResourceStatus, payload: Option<serde_json::Value>) -> ResourceData {
        ResourceData {
            status,
            payload,
            last_synced: Utc::now(),
        }
    }

    pub fn set_status(&mut self, status: ResourceStatus) {
        self.status = status;
    }
}

impl From<&ActionRunResultSuccess> for ResourceData {
    fn from(value: &ActionRunResultSuccess) -> Self {
        ResourceData::new(value.status, value.payload.clone())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceView {
    pub status: Option<ResourceStatus>,
    pub message: Option<String>,
    pub payload: Option<Value>,
    pub last_synced: Option<String>,
}

impl ResourceView {
    pub async fn get_by_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Self> {
        let component = Component::get_by_id(ctx, component_id).await?;

        let resource = Self::assemble(component.resource(ctx).await?);
        Ok(resource)
    }

    pub fn assemble(maybe_result: Option<ResourceData>) -> Self {
        match maybe_result {
            Some(result) => Self {
                payload: result.payload,
                message: None,
                status: Some(result.status),
                last_synced: Some(result.last_synced.to_string()),
            },
            None => Self {
                payload: None,
                message: None,
                status: None,
                last_synced: None,
            },
        }
    }
}

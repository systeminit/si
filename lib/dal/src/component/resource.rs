//! This module contains the ability to work with "resources" for [`Components`](crate::Component).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use veritech_client::ResourceStatus;

use crate::component::ComponentResult;
use crate::func::backend::js_action::DeprecatedActionRunResult;
use crate::{Component, ComponentId, DalContext};

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

        let resource = Self::assemble(component.resource(ctx).await?)?;
        Ok(resource)
    }

    pub fn assemble(result: DeprecatedActionRunResult) -> ComponentResult<Self> {
        let payload: Value = match result.payload {
            Some(payload) => payload,
            None => Value::Null,
        };

        Ok(Self {
            payload: Some(payload),
            message: result.message,
            status: result.status,
            last_synced: result.last_synced,
        })
    }
}

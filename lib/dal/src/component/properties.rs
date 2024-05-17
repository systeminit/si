//! This module provides [`ComponentProperties`], which is a builder-pattern struct that enables
//! users to modify an existing component safely.

use crate::ComponentError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ComponentProperties {
    pub(crate) si: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) domain: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) resource: Option<ResourceProperties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) code: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) qualification: Option<serde_json::Value>,
}

/// This _private_ struct provides the ability to drop fields for the "/root/resource" tree at a
/// more granular level than [`ComponentViewProperties`].
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ResourceProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_synced: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<serde_json::Value>,
}

impl ComponentProperties {
    pub fn drop_private(&mut self) -> &mut Self {
        *self = Self {
            si: self.si.take(),
            domain: self.domain.take(),
            ..Default::default()
        };
        self
    }

    /// Drops the value corresponding to "/root/code".
    pub fn drop_code(&mut self) -> &mut Self {
        self.code = None;
        self
    }

    /// Drops the value corresponding to "/root/qualification".
    pub fn drop_qualification(&mut self) -> &mut Self {
        self.qualification = None;
        self
    }

    /// Drops the value corresponding to "/root/resource/last_synced".
    pub fn drop_resource_last_synced(&mut self) -> &mut Self {
        if let Some(mut resource) = self.resource.clone() {
            resource.last_synced = None;
            self.resource = Some(resource);
        }
        self
    }
}

impl TryFrom<serde_json::Value> for ComponentProperties {
    type Error = ComponentError;

    fn try_from(view: Value) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(view)?)
    }
}

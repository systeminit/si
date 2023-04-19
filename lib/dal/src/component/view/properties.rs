//! This module provides [`ComponentViewProperties`], which is a builder-pattern struct that enables
//! users to modify an existing [`ComponentView`] safely.

use serde::{Deserialize, Serialize};

use crate::component::view::ComponentViewResult;
use crate::component::ComponentViewError;
use crate::ComponentView;

/// This struct provides the ability to drop fields from a [`ComponentView`](crate::ComponentView)
/// properties tree and then re-render the view using [`Self::to_value()`].
///
/// - It is not recommended to use [`self`] "as-is" in assertions.
/// - It is recommended to use [`Self::to_value()`] in assertions.
///
/// The fields on this struct are **intentionally private**.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ComponentViewProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    si: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<ResourceProperties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qualification: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirmation: Option<serde_json::Value>,
}

/// This _private_ struct provides the ability to drop fields for the "/root/resource" tree at a
/// more granular level than [`ComponentViewProperties`].
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
struct ResourceProperties {
    status: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logs: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_synced: Option<serde_json::Value>,
}

impl ComponentViewProperties {
    /// Create a new [`ComponentViewProperties`] object by using [`Self::try_from`] with a
    /// [`ComponentView`].
    pub fn new(view: ComponentView) -> ComponentViewResult<Self> {
        Self::try_from(view)
    }

    pub fn drop_all_but_domain(&mut self) -> &mut Self {
        *self = Self {
            domain: self.domain.take(),
            ..Default::default()
        };
        self
    }

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

    /// Drops the value corresponding to "/root/confirmation".
    pub fn drop_confirmation(&mut self) -> &mut Self {
        self.confirmation = None;
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

    /// Converts [`self`](ComponentViewProperties) into a serialized [`Value`](serde_json::Value).
    pub fn to_value(&self) -> ComponentViewResult<serde_json::Value> {
        let value = serde_json::to_value(self)?;
        Ok(value)
    }
}

impl TryFrom<ComponentView> for ComponentViewProperties {
    type Error = ComponentViewError;

    fn try_from(view: ComponentView) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(view.properties)?)
    }
}

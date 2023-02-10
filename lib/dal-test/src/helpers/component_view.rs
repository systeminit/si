use dal::ComponentView;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

/// This struct provides the ability to drop fields from a [`ComponentView`](dal::ComponentView)
/// properties tree and then re-render the view using [`Self::to_value()`].
///
/// - It is not recommended to use [`self`] "as-is" in assertions.
/// - It is recommended to use [`Self::to_value()`] in assertions.
///
/// The fields on this struct are **intentionally private**.
#[derive(Deserialize, Serialize, Debug)]
pub struct ComponentViewProperties {
    si: serde_json::Value,
    domain: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qualification: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confirmation: Option<serde_json::Value>,
}

#[derive(Error, Debug)]
pub enum ComponentViewPropertiesError {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

impl ComponentViewProperties {
    pub fn drop_code(&mut self) -> &mut Self {
        self.code = None;
        self
    }

    pub fn drop_qualification(&mut self) -> &mut Self {
        self.qualification = None;
        self
    }

    pub fn drop_confirmation(&mut self) -> &mut Self {
        self.confirmation = None;
        self
    }

    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("could not serialize into value")
    }
}

impl TryFrom<ComponentView> for ComponentViewProperties {
    type Error = ComponentViewPropertiesError;

    fn try_from(value: ComponentView) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(value.properties)?)
    }
}

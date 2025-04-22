//! This module contains the ability to switch a [`Component's`](crate::Component) type between
//! a standard [`Component`](crate::Component) and a "frame". This functionality resides in this
//! location because it corresponds to the "/root/si/type" location in the
//! [`RootProp`](crate::RootProp) tree.
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_types as frontend_types;
use si_pkg::SchemaVariantSpecComponentType;
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

/// The possible values of "/root/si/type".
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub enum ComponentType {
    #[serde(alias = "AggregationFrame")]
    #[strum(serialize = "AggregationFrame", serialize = "aggregationFrame")]
    AggregationFrame,
    #[serde(alias = "Component")]
    #[strum(serialize = "Component", serialize = "component")]
    #[default]
    Component,
    #[serde(
        alias = "ConfigurationFrameDown",
        alias = "ConfigurationFrame",
        alias = "configurationFrame"
    )]
    #[strum(
        serialize = "ConfigurationFrameDown",
        serialize = "configurationFrameDown",
        serialize = "ConfigurationFrame",
        serialize = "configurationFrame"
    )]
    ConfigurationFrameDown,
    #[strum(serialize = "ConfigurationFrameUp", serialize = "configurationFrameUp")]
    ConfigurationFrameUp,
}

impl From<SchemaVariantSpecComponentType> for ComponentType {
    fn from(value: SchemaVariantSpecComponentType) -> Self {
        match value {
            SchemaVariantSpecComponentType::Component => Self::Component,
            SchemaVariantSpecComponentType::AggregationFrame => Self::AggregationFrame,
            SchemaVariantSpecComponentType::ConfigurationFrameDown => Self::ConfigurationFrameDown,
            SchemaVariantSpecComponentType::ConfigurationFrameUp => Self::ConfigurationFrameUp,
        }
    }
}

impl From<ComponentType> for SchemaVariantSpecComponentType {
    fn from(value: ComponentType) -> Self {
        match value {
            ComponentType::Component => Self::Component,
            ComponentType::AggregationFrame => Self::AggregationFrame,
            ComponentType::ConfigurationFrameDown => Self::ConfigurationFrameDown,
            ComponentType::ConfigurationFrameUp => Self::ConfigurationFrameUp,
        }
    }
}

impl From<frontend_types::ComponentType> for ComponentType {
    fn from(value: frontend_types::ComponentType) -> Self {
        match value {
            si_frontend_types::ComponentType::AggregationFrame => Self::AggregationFrame,
            si_frontend_types::ComponentType::Component => Self::Component,
            si_frontend_types::ComponentType::ConfigurationFrameDown => {
                Self::ConfigurationFrameDown
            }
            si_frontend_types::ComponentType::ConfigurationFrameUp => Self::ConfigurationFrameUp,
        }
    }
}

impl From<ComponentType> for frontend_types::ComponentType {
    fn from(value: ComponentType) -> Self {
        match value {
            ComponentType::AggregationFrame => Self::AggregationFrame,
            ComponentType::Component => Self::Component,
            ComponentType::ConfigurationFrameDown => Self::ConfigurationFrameDown,
            ComponentType::ConfigurationFrameUp => Self::ConfigurationFrameUp,
        }
    }
}

impl ComponentType {
    /// Return the label corresponding to [`self`](Self).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Component => "Component",
            Self::AggregationFrame => "Aggregation Frame",
            Self::ConfigurationFrameDown => "Configuration Frame (down)",
            Self::ConfigurationFrameUp => "Configuration Frame (up)",
        }
    }

    pub fn is_frame(&self) -> bool {
        matches!(
            self,
            Self::AggregationFrame | Self::ConfigurationFrameDown | Self::ConfigurationFrameUp
        )
    }
}

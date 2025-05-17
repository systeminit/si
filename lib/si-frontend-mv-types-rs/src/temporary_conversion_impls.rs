//! This module contains conversions from dal-reliant frontend types that should be exclusively
//! living in this crate for MV generation.
//!
//! This module is a hold-over until the new UI is in place and the old UI (alongside its routes
//! using the old types) is dropped.

use si_frontend_types::{
    ComponentQualificationStats as DeprecatedComponentQualificationStats,
    ComponentType as DeprecatedComponentType,
    InputSocket as DeprecatedInputSocket,
    OutputSocket as DeprecatedOutputSocket,
    PropKind as DeprecatedPropKind,
};

use crate::{
    ComponentType,
    InputSocket,
    OutputSocket,
    component::ComponentQualificationStats,
    schema_variant::prop_tree::PropKind,
};

impl From<DeprecatedComponentQualificationStats> for ComponentQualificationStats {
    fn from(value: DeprecatedComponentQualificationStats) -> Self {
        Self {
            total: value.total,
            warned: value.warned,
            succeeded: value.succeeded,
            failed: value.failed,
            running: value.running,
        }
    }
}

// NOTE(nick): why are we converting component type if it is dead in the new UI? Baggage from the
// old type in the new MV. For now, let's convert it but the concept of "component type" will also
// die alongside this module.
impl From<DeprecatedComponentType> for ComponentType {
    fn from(value: DeprecatedComponentType) -> Self {
        match value {
            DeprecatedComponentType::AggregationFrame => Self::AggregationFrame,
            DeprecatedComponentType::Component => Self::Component,
            DeprecatedComponentType::ConfigurationFrameDown => Self::ConfigurationFrameDown,
            DeprecatedComponentType::ConfigurationFrameUp => Self::ConfigurationFrameUp,
        }
    }
}

impl From<DeprecatedInputSocket> for InputSocket {
    fn from(value: DeprecatedInputSocket) -> Self {
        Self {
            id: value.id,
            name: value.name,
            eligible_to_send_data: value.eligible_to_send_data,
            annotations: Vec::new(),
            arity: String::new(),
        }
    }
}

impl From<DeprecatedOutputSocket> for OutputSocket {
    fn from(value: DeprecatedOutputSocket) -> Self {
        Self {
            id: value.id,
            name: value.name,
            eligible_to_receive_data: value.eligible_to_receive_data,
            annotations: Vec::new(),
            arity: String::new(),
        }
    }
}

impl From<DeprecatedPropKind> for PropKind {
    fn from(value: DeprecatedPropKind) -> Self {
        match value {
            DeprecatedPropKind::Any => Self::Any,
            DeprecatedPropKind::Array => Self::Array,
            DeprecatedPropKind::Boolean => Self::Boolean,
            DeprecatedPropKind::Float => Self::Float,
            DeprecatedPropKind::Integer => Self::Integer,
            DeprecatedPropKind::Json => Self::Json,
            DeprecatedPropKind::Map => Self::Map,
            DeprecatedPropKind::Object => Self::Object,
            DeprecatedPropKind::String => Self::String,
        }
    }
}

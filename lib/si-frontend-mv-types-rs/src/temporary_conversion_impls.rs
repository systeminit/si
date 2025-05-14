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
    Prop as DeprecatedProp,
    PropKind as DeprecatedPropKind,
    SchemaVariant as DeprecatedSchemaVariant,
};

use crate::{
    ComponentType,
    InputSocket,
    OutputSocket,
    Prop,
    PropKind,
    SchemaVariant,
    component::ComponentQualificationStats,
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

impl From<DeprecatedSchemaVariant> for SchemaVariant {
    fn from(value: DeprecatedSchemaVariant) -> Self {
        Self {
            id: value.schema_variant_id,
            schema_id: value.schema_id,
            schema_name: value.schema_name,
            schema_variant_id: value.schema_variant_id,
            version: value.version,
            display_name: value.display_name,
            category: value.category,
            description: value.description,
            link: value.link,
            color: value.color,
            input_sockets: value.input_sockets.into_iter().map(Into::into).collect(),
            output_sockets: value.output_sockets.into_iter().map(Into::into).collect(),
            props: value.props.into_iter().map(Into::into).collect(),
            is_locked: value.is_locked,
            timestamp: value.timestamp,
            can_create_new_components: value.can_create_new_components,
            can_contribute: value.can_contribute,
            mgmt_functions: [].to_vec(),
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

impl From<DeprecatedProp> for Prop {
    fn from(value: DeprecatedProp) -> Self {
        Self {
            id: value.id,
            kind: value.kind.into(),
            name: value.name,
            path: value.path,
            hidden: value.hidden,
            eligible_to_receive_data: value.eligible_to_receive_data,
            eligible_to_send_data: value.eligible_to_send_data,
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

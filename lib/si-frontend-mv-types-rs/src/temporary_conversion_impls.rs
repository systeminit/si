//! This module contains conversions from dal-reliant frontend types that should be exclusively
//! living in this crate for MV generation.
//!
//! This module is a hold-over until the new UI is in place and the old UI (alongside its routes
//! using the old types) is dropped.

use si_frontend_types::{
    ComponentQualificationStats as DeprecatedComponentQualificationStats,
    PropKind as DeprecatedPropKind,
};

use crate::{
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

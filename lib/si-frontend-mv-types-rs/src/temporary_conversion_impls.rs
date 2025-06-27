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
        // NOTE(nick): notice how "value.running" isn't called... we need it in the deprecated type
        // for the old UI, but we don't need it in the new UI. It actually shouldn't be in either,
        // but we do not want to regress the old UI by accident.
        Self {
            total: value.total,
            warned: value.warned,
            succeeded: value.succeeded,
            failed: value.failed,
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

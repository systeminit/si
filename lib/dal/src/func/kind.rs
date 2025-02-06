use serde::{Deserialize, Serialize};
use si_events::FuncKind as EventFuncKind;
use strum::{AsRefStr, Display};
use telemetry::prelude::warn;

use crate::func::FuncResult;
use crate::{FuncBackendKind, FuncBackendResponseType};

/// Describes the kind of [`Func`](crate::Func).
/// This type is postcard serialized, so cannot be "remain::sorted". New enum
/// variants must go at the end
#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum FuncKind {
    Action,
    Attribute,
    Authentication,
    CodeGeneration,
    Intrinsic,
    Qualification,
    SchemaVariantDefinition,
    Unknown,
    Management,
}

impl From<EventFuncKind> for FuncKind {
    fn from(value: EventFuncKind) -> Self {
        match value {
            EventFuncKind::Action => FuncKind::Action,
            EventFuncKind::Attribute => FuncKind::Attribute,
            EventFuncKind::Authentication => FuncKind::Authentication,
            EventFuncKind::CodeGeneration => FuncKind::CodeGeneration,
            EventFuncKind::Intrinsic => FuncKind::Intrinsic,
            EventFuncKind::Qualification => FuncKind::Qualification,
            EventFuncKind::SchemaVariantDefinition => FuncKind::SchemaVariantDefinition,
            EventFuncKind::Unknown => FuncKind::Unknown,
            EventFuncKind::Management => FuncKind::Management,
        }
    }
}

impl From<FuncKind> for si_events::FuncKind {
    fn from(value: FuncKind) -> Self {
        match value {
            FuncKind::Action => si_events::FuncKind::Action,
            FuncKind::Attribute => si_events::FuncKind::Attribute,
            FuncKind::Authentication => si_events::FuncKind::Authentication,
            FuncKind::CodeGeneration => si_events::FuncKind::CodeGeneration,
            FuncKind::Intrinsic => si_events::FuncKind::Intrinsic,
            FuncKind::Qualification => si_events::FuncKind::Qualification,
            FuncKind::SchemaVariantDefinition => si_events::FuncKind::SchemaVariantDefinition,
            FuncKind::Unknown => si_events::FuncKind::Unknown,
            FuncKind::Management => si_events::FuncKind::Management,
        }
    }
}

impl FuncKind {
    pub fn new(
        func_backend_kind: FuncBackendKind,
        func_backend_response_type: FuncBackendResponseType,
    ) -> FuncResult<FuncKind> {
        Ok(match func_backend_kind {
            FuncBackendKind::JsAttribute => match func_backend_response_type {
                FuncBackendResponseType::CodeGeneration => FuncKind::CodeGeneration,
                FuncBackendResponseType::Qualification => FuncKind::Qualification,
                _ => FuncKind::Attribute,
            },
            FuncBackendKind::JsAction => FuncKind::Action,
            FuncBackendKind::JsAuthentication => FuncKind::Authentication,
            FuncBackendKind::JsSchemaVariantDefinition => FuncKind::SchemaVariantDefinition,
            FuncBackendKind::Management => FuncKind::Management,
            FuncBackendKind::Array
            | FuncBackendKind::Json
            | FuncBackendKind::Boolean
            | FuncBackendKind::Diff
            | FuncBackendKind::Identity
            | FuncBackendKind::Integer
            | FuncBackendKind::Map
            | FuncBackendKind::NormalizeToArray
            | FuncBackendKind::Object
            | FuncBackendKind::ResourcePayloadToValue
            | FuncBackendKind::String
            | FuncBackendKind::Unset
            | FuncBackendKind::Validation => FuncKind::Intrinsic,
            FuncBackendKind::JsReconciliation | FuncBackendKind::JsValidation => {
                warn!(
                    %func_backend_kind,
                    %func_backend_response_type,
                    "found deprecated or unknown func backend kind, marking as unknown"
                );
                FuncKind::Unknown
            }
        })
    }
}

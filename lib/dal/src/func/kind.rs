use serde::{
    Deserialize,
    Serialize,
};
use si_events::FuncKind as EventFuncKind;
use strum::{
    AsRefStr,
    Display,
};
use telemetry::prelude::warn;

use crate::{
    FuncBackendKind,
    FuncBackendResponseType,
    func::FuncResult,
};

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
    Debug,
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
            EventFuncKind::Debug => FuncKind::Debug,
        }
    }
}

impl From<FuncKind> for si_events::FuncKind {
    fn from(value: FuncKind) -> Self {
        match value {
            FuncKind::Action => EventFuncKind::Action,
            FuncKind::Attribute => EventFuncKind::Attribute,
            FuncKind::Authentication => EventFuncKind::Authentication,
            FuncKind::CodeGeneration => EventFuncKind::CodeGeneration,
            FuncKind::Intrinsic => EventFuncKind::Intrinsic,
            FuncKind::Qualification => EventFuncKind::Qualification,
            FuncKind::SchemaVariantDefinition => EventFuncKind::SchemaVariantDefinition,
            FuncKind::Unknown => EventFuncKind::Unknown,
            FuncKind::Management => EventFuncKind::Management,
            FuncKind::Debug => EventFuncKind::Debug,
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
            FuncBackendKind::Debug => FuncKind::Debug,
            FuncBackendKind::Array
            | FuncBackendKind::Json
            | FuncBackendKind::Boolean
            | FuncBackendKind::Diff
            | FuncBackendKind::Identity
            | FuncBackendKind::Float
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

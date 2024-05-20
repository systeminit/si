use serde::{Deserialize, Serialize};
use si_events::FuncKind as EventFuncKind;
use strum::{AsRefStr, Display};
use telemetry::prelude::warn;

use crate::func::FuncResult;
use crate::{FuncBackendKind, FuncBackendResponseType, FuncError};

/// Describes the kind of [`Func`](crate::Func).
#[remain::sorted]
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
        }
    }
}

impl FuncKind {
    pub fn new(
        func_backend_kind: FuncBackendKind,
        func_backend_response_type: FuncBackendResponseType,
    ) -> FuncResult<FuncKind> {
        match func_backend_kind {
            FuncBackendKind::JsAttribute => match func_backend_response_type {
                FuncBackendResponseType::CodeGeneration => Ok(FuncKind::CodeGeneration),
                FuncBackendResponseType::Qualification => Ok(FuncKind::Qualification),
                _ => Ok(FuncKind::Attribute),
            },
            FuncBackendKind::JsAction => Ok(FuncKind::Action),
            FuncBackendKind::JsAuthentication => Ok(FuncKind::Authentication),
            FuncBackendKind::JsSchemaVariantDefinition => Ok(FuncKind::SchemaVariantDefinition),
            FuncBackendKind::JsValidation => {
                warn!(
                    ?func_backend_kind,
                    ?func_backend_response_type,
                    "found JsValidation func backend kind, marking as unknown"
                );
                Ok(FuncKind::Unknown)
            }
            FuncBackendKind::Array
            | FuncBackendKind::Json
            | FuncBackendKind::Boolean
            | FuncBackendKind::Diff
            | FuncBackendKind::Identity
            | FuncBackendKind::Integer
            | FuncBackendKind::Map
            | FuncBackendKind::Object
            | FuncBackendKind::String
            | FuncBackendKind::Unset
            | FuncBackendKind::Validation => Ok(FuncKind::Intrinsic),
            _ => Err(FuncError::UnknownFunctionType(
                func_backend_kind,
                func_backend_response_type,
            )),
        }
    }
}

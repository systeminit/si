use serde::{Deserialize, Serialize};
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

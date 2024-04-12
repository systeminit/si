//! This module contains the definition of the Function Variant struct
//! which represents a function for the frontend. It doesn't map 1:1 onto FuncBackendKind,
//! since some JsAttribute functions are a special case (Qualification, CodeGeneration etc.)

use crate::{Func, FuncBackendKind, FuncBackendResponseType, FuncError};
use serde::{Deserialize, Serialize};

#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum FuncVariant {
    Action,
    Attribute,
    Authentication,
    CodeGeneration,
    Qualification,
    Reconciliation,
    Validation,
}

impl From<FuncVariant> for FuncBackendKind {
    fn from(value: FuncVariant) -> Self {
        match value {
            FuncVariant::Reconciliation => FuncBackendKind::JsReconciliation,
            FuncVariant::Action => FuncBackendKind::JsAction,
            FuncVariant::Validation => FuncBackendKind::JsValidation,
            FuncVariant::Attribute | FuncVariant::CodeGeneration | FuncVariant::Qualification => {
                FuncBackendKind::JsAttribute
            }
            FuncVariant::Authentication => FuncBackendKind::JsAuthentication,
        }
    }
}

impl TryFrom<&Func> for FuncVariant {
    type Error = FuncError;

    fn try_from(func: &Func) -> Result<Self, Self::Error> {
        match (func.backend_kind(), func.backend_response_type()) {
            (FuncBackendKind::JsAttribute, response_type) => match response_type {
                FuncBackendResponseType::CodeGeneration => Ok(FuncVariant::CodeGeneration),
                FuncBackendResponseType::Qualification => Ok(FuncVariant::Qualification),
                _ => Ok(FuncVariant::Attribute),
            },
            (FuncBackendKind::JsReconciliation, _) => Ok(FuncVariant::Reconciliation),
            (FuncBackendKind::JsAction, _) => Ok(FuncVariant::Action),
            (FuncBackendKind::JsValidation, _) => Ok(FuncVariant::Validation),
            (FuncBackendKind::JsAuthentication, _) => Ok(FuncVariant::Authentication),
            (FuncBackendKind::Array, _)
            | (FuncBackendKind::Boolean, _)
            | (FuncBackendKind::Diff, _)
            | (FuncBackendKind::Identity, _)
            | (FuncBackendKind::Integer, _)
            | (FuncBackendKind::JsSchemaVariantDefinition, _)
            | (FuncBackendKind::Map, _)
            | (FuncBackendKind::Object, _)
            | (FuncBackendKind::String, _)
            | (FuncBackendKind::Unset, _)
            | (FuncBackendKind::Validation, _) => {
                Err(FuncError::FuncCannotBeTurnedIntoVariant(func.id))
            }
        }
    }
}

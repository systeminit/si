//! This module contains backend functionality for the [`Func`] authoring experience.
//!
//! All submodules are private since the [`FuncAuthoringClient`] is the primary interface.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::AttributePrototypeError;
use crate::func::argument::FuncArgumentError;
use crate::func::associations::FuncAssociations;
use crate::func::view::FuncViewError;
use crate::func::FuncKind;
use crate::{
    DalContext, DeprecatedActionKind, DeprecatedActionPrototypeError, Func, FuncBackendKind,
    FuncBackendResponseType, FuncError, FuncId, OutputSocketId, PropId, SchemaVariantError,
    SchemaVariantId,
};

mod create;
mod save;
mod types;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncAuthoringError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] DeprecatedActionPrototypeError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("That attribute is already set by the function named \"{0}\"")]
    AttributePrototypeAlreadySetByFunc(String),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func named \"{0}\" already exists in this change set")]
    FuncNameExists(String),
    #[error("Function options are incompatible with variant")]
    FuncOptionsAndVariantMismatch,
    #[error("func view error: {0}")]
    FuncView(#[from] FuncViewError),
    #[error("invalid func kind for creation: {0}")]
    InvalidFuncKindForCreation(FuncKind),
    #[error("func is read-only")]
    NotWritable,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("unexpected func kind ({0}) creating attribute func")]
    UnexpectedFuncKindCreatingAttributeFunc(FuncKind),
}

type FuncAuthoringResult<T> = Result<T, FuncAuthoringError>;

/// This unit struct is the primary interface for the [`Func`](crate::Func) authoring experience.
#[derive(Debug)]
pub struct FuncAuthoringClient;

impl FuncAuthoringClient {
    /// Creates a [`Func`] and returns the [result](CreatedFunc).
    #[instrument(name = "func.authoring.create_func", level = "info", skip_all)]
    pub async fn create_func(
        ctx: &DalContext,
        kind: FuncKind,
        name: Option<String>,
        options: Option<CreateFuncOptions>,
    ) -> FuncAuthoringResult<CreatedFunc> {
        create::create_func(ctx, kind, name, options).await
    }

    /// Saves a [`Func`] and returns the [result](SavedFunc).
    #[instrument(name = "func.authoring.save_func", level = "info", skip_all)]
    pub async fn save_func(
        ctx: &DalContext,
        id: FuncId,
        display_name: Option<String>,
        name: String,
        description: Option<String>,
        code: Option<String>,
        associations: Option<FuncAssociations>,
    ) -> FuncAuthoringResult<(SavedFunc, Func)> {
        save::save_func(ctx, id, display_name, name, description, code, associations).await
    }

    /// Compiles types corresponding to "lang-js".
    pub fn compile_langjs_types() -> &'static str {
        types::compile_langjs_types()
    }

    /// Compiles return types based on a [`FuncBackendResponseType`] and [`FuncBackendKind`].
    pub fn compile_return_types(
        response_type: FuncBackendResponseType,
        kind: FuncBackendKind,
    ) -> &'static str {
        types::compile_return_types(response_type, kind)
    }
}

/// The result of creating a [`Func`] via [`FuncAuthoringClient::create_func`].
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatedFunc {
    /// The id of the created [`Func`].
    pub id: FuncId,
    /// The handler of the created [`Func`].
    pub handler: Option<String>,
    /// The [kind](FuncKind) of the created [`Func`].
    pub kind: FuncKind,
    /// The name of the created [`Func`].
    pub name: String,
    /// The code for the created [`Func`].
    pub code: Option<String>,
}

/// The result of creating a [`Func`] via [`FuncAuthoringClient::save_func`].
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SavedFunc {
    /// The [associations](FuncAssociations) for a saved [`Func`].
    pub associations: Option<FuncAssociations>,
    /// Indicates the success of saving the [`Func`].
    pub success: bool,
    /// Indicates if the [`Func`] is ["revertible"](Func::is_revertible).
    pub is_revertible: bool,
    /// The compiled types for the saved [`Func`].
    pub types: String,
}

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AttributeOutputLocation {
    #[serde(rename_all = "camelCase")]
    OutputSocket { output_socket_id: OutputSocketId },
    #[serde(rename_all = "camelCase")]
    Prop { prop_id: PropId },
}

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CreateFuncOptions {
    #[serde(rename_all = "camelCase")]
    ActionOptions {
        schema_variant_id: SchemaVariantId,
        action_kind: DeprecatedActionKind,
    },
    #[serde(rename_all = "camelCase")]
    AttributeOptions {
        schema_variant_id: SchemaVariantId,
        output_location: AttributeOutputLocation,
    },
    #[serde(rename_all = "camelCase")]
    AuthenticationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    CodeGenerationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    QualificationOptions { schema_variant_id: SchemaVariantId },
}

// /// Determines what we should do with the [`AttributePrototype`](dal::AttributePrototype) and
// /// [`AttributeValues`](dal::AttributeValue) that are currently associated with a function but
// /// that are having their association removed.
// ///
// /// `RemovedPrototypeOp::Reset` takes the currenty value and resets the prototype to set it to that
// /// value using a builtin value function, like `si:setString`, etc.
// ///
// /// `RemovedPrototypeOp::Delete` deletes the prototype and its values.
// #[remain::sorted]
// #[derive(Debug)]
// enum RemovedPrototypeOp {
//     Delete,
//     Reset,
// }

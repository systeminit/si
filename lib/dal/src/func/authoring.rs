//! This module contains backend functionality for the [`Func`] authoring experience.
//!
//! How does the authoring loop work? While metadata fetching and mutation is self-explanatory, the
//! [`FuncAssociations`] subsystem is less so. Essentially, [`FuncAssociations`] are a "bag" that
//! come alongside [`FuncView::assemble`](crate::func::view::FuncView::assemble) and are mutated via
//! [`FuncAuthoringClient::save_func`].
//!
//! The existence, difference or absence of an entity or field within a [`FuncAssociations`] bag
//! dictates what we must do during mutation. New argument? Create one and send it back up. Removed
//! argument? Delete it and send it back up. Mutated argument? Change it and send it back up. The
//! payload is the same on both sides.
//!
//! _Note_: all submodules are private since the [`FuncAuthoringClient`] is the primary interface.

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
use veritech_client::OutputStream;

use crate::attribute::prototype::argument::{
    AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::prototype::AttributePrototypeError;
use crate::attribute::value::AttributeValueError;
use crate::func::argument::{FuncArgumentError, FuncArgumentId};
use crate::func::associations::{FuncAssociations, FuncAssociationsError};
use crate::func::binding::FuncBindingError;
use crate::func::view::FuncViewError;
use crate::func::FuncKind;
use crate::prop::PropError;
use crate::secret::BeforeFuncError;
use crate::socket::output::OutputSocketError;
use crate::{
    AttributePrototypeId, ComponentError, ComponentId, DalContext, DeprecatedActionKind,
    DeprecatedActionPrototypeError, FuncBackendKind, FuncBackendResponseType, FuncError, FuncId,
    OutputSocketId, PropId, SchemaVariantError, SchemaVariantId, TransactionsError, WsEventError,
};

mod create;
mod execute;
mod save;
mod test_execute;
mod ts_types;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncAuthoringError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] DeprecatedActionPrototypeError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype already set by func (id: {0}) (name: {1})")]
    AttributePrototypeAlreadySetByFunc(FuncId, String),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value not found for attribute prototype: {0}")]
    AttributeValueNotFoundForAttributePrototype(AttributePrototypeId),
    #[error("before func error: {0}")]
    BeforeFunc(#[from] BeforeFuncError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func argument must exist before using it in an attribute prototype argument: {0}")]
    FuncArgumentMustExist(AttributePrototypeArgumentId),
    #[error("func associations error: {0}")]
    FuncAssociations(#[from] FuncAssociationsError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func named \"{0}\" already exists in this change set")]
    FuncNameExists(String),
    #[error("Function options are incompatible with variant")]
    FuncOptionsAndVariantMismatch,
    #[error("func view error: {0}")]
    FuncView(#[from] FuncViewError),
    #[error("invalid func kind for creation: {0}")]
    InvalidFuncKindForCreation(FuncKind),
    #[error("no input location given for attribute prototype id ({0}) and func argument id ({1})")]
    NoInputLocationGiven(AttributePrototypeId, FuncArgumentId),
    #[error("no output location given for func: {0}")]
    NoOutputLocationGiven(FuncId),
    #[error("func ({0}) is not runnable with kind: {1}")]
    NotRunnable(FuncId, FuncKind),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("tokio task join error: {0}")]
    TokioTaskJoin(#[from] tokio::task::JoinError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("unexpected func kind ({0}) creating attribute func")]
    UnexpectedFuncKindCreatingAttributeFunc(FuncKind),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
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

    /// Performs a "test" [`Func`] execution and returns the [result](TestExecuteFuncResult).
    #[instrument(name = "func.authoring.test_execute_func", level = "info", skip_all)]
    pub async fn test_execute_func(
        ctx: &DalContext,
        id: FuncId,
        args: serde_json::Value,
        execution_key: String,
        code: String,
        component_id: ComponentId,
    ) -> FuncAuthoringResult<TestExecuteFuncResult> {
        test_execute::test_execute_func(ctx, id, args, execution_key, code, component_id).await
    }

    /// Executes a [`Func`].
    #[instrument(name = "func.authoring.execute_func", level = "info", skip_all)]
    pub async fn execute_func(ctx: &DalContext, id: FuncId) -> FuncAuthoringResult<()> {
        execute::execute_func(ctx, id).await
    }

    /// Saves a [`Func`].
    #[instrument(name = "func.authoring.save_func", level = "info", skip_all)]
    pub async fn save_func(
        ctx: &DalContext,
        id: FuncId,
        display_name: Option<String>,
        name: String,
        description: Option<String>,
        code: Option<String>,
        associations: Option<FuncAssociations>,
    ) -> FuncAuthoringResult<()> {
        save::save_func(ctx, id, display_name, name, description, code, associations).await
    }

    /// Compiles types corresponding to "lang-js".
    pub fn compile_langjs_types() -> &'static str {
        ts_types::compile_langjs_types()
    }

    /// Compiles return types based on a [`FuncBackendResponseType`] and [`FuncBackendKind`].
    pub fn compile_return_types(
        response_type: FuncBackendResponseType,
        kind: FuncBackendKind,
    ) -> &'static str {
        ts_types::compile_return_types(response_type, kind)
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

/// The result of a [`test execution`](FuncAuthoringClient::dummy_execute).
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestExecuteFuncResult {
    /// The ID of the [`Func`](crate::Func) that was "test" executed.
    pub id: FuncId,
    /// The serialized arguments provided as inputs to the [`Func`](crate::Func) for test execution.
    pub args: serde_json::Value,
    /// The serialized output of the test execution.
    pub output: serde_json::Value,
    /// The key for the test execution (e.g. a randomized string that the user keeps track of).
    pub execution_key: String,
    /// The logs corresponding to the output stream of the test execution.
    pub logs: Vec<OutputStream>,
}

/// Determines what we should do with the [`AttributePrototype`](dal::AttributePrototype) and
/// [`AttributeValues`](dal::AttributeValue) that are currently associated with a function but
/// that are having their association removed.
///
/// `RemovedPrototypeOp::Reset` takes the currenty value and resets the prototype to set it to that
/// value using a builtin value function, like `si:setString`, etc.
///
/// `RemovedPrototypeOp::Delete` deletes the prototype and its values.
#[remain::sorted]
#[derive(Debug, Copy, Clone)]
enum RemovedPrototypeOp {
    Delete,
    Reset,
}

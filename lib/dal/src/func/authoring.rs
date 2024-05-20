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

use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};
use si_events::FuncRunId;
use telemetry::prelude::*;
use thiserror::Error;

use crate::action::prototype::{ActionKind, ActionPrototypeError};
use crate::attribute::prototype::argument::{
    AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::prototype::AttributePrototypeError;
use crate::attribute::value::AttributeValueError;
use crate::func::argument::{FuncArgumentError, FuncArgumentId};
use crate::func::associations::{FuncAssociations, FuncAssociationsError};
use crate::func::view::FuncViewError;
use crate::func::FuncKind;
use crate::prop::PropError;
use crate::secret::BeforeFuncError;
use crate::socket::output::OutputSocketError;
use crate::{
    AttributePrototypeId, ComponentError, ComponentId, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncError, FuncId, OutputSocketId, PropId, SchemaVariantError,
    SchemaVariantId, TransactionsError, WorkspaceSnapshotError, WsEventError,
};

use super::runner::FuncRunnerError;

mod create;
mod execute;
mod save;
mod test_execute;
mod ts_types;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncAuthoringError {
    #[error("action kind already exists for schema variant")]
    ActionKindAlreadyExists(SchemaVariantId),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype already set by func (id: {0}) (name: {1})")]
    AttributePrototypeAlreadySetByFunc(FuncId, String),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
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
    #[error("func ({0}) with kind ({1}) cannot have associations: {2:?}")]
    FuncCannotHaveAssociations(FuncId, FuncKind, FuncAssociations),
    #[error("func named \"{0}\" already exists in this change set")]
    FuncNameExists(String),
    #[error("Function options are incompatible with variant")]
    FuncOptionsAndVariantMismatch,
    #[error("func run value sender is gone without sending a value")]
    FuncRunGone,
    #[error("func run error: {0}")]
    FuncRunner(#[from] FuncRunnerError),
    #[error("func view error: {0}")]
    FuncView(#[from] FuncViewError),
    #[error("invalid func associations ({0:?}) for func ({1}) of kind: {2}")]
    InvalidFuncAssociationsForFunc(FuncAssociations, FuncId, FuncKind),
    #[error("invalid func kind for creation: {0}")]
    InvalidFuncKindForCreation(FuncKind),
    #[error("can't use func kind '{0}' as it already exists")]
    KindAlreadyExists(ActionKind),
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
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

type FuncAuthoringResult<T> = Result<T, FuncAuthoringError>;

/// This unit struct is the primary interface for the [`Func`](crate::Func) authoring experience.
#[derive(Debug)]
pub struct FuncAuthoringClient;

impl FuncAuthoringClient {
    /// Creates a [`Func`] and returns the [result](CreatedFunc).
    #[instrument(name = "func.authoring.create_func", level = "info", skip(ctx))]
    pub async fn create_func(
        ctx: &DalContext,
        kind: FuncKind,
        name: Option<String>,
        options: Option<CreateFuncOptions>,
    ) -> FuncAuthoringResult<CreatedFunc> {
        let func = create::create(ctx, kind, name, options).await?;
        Ok(CreatedFunc {
            id: func.id,
            handler: func.handler.as_ref().map(|h| h.to_owned()),
            kind: func.kind,
            name: func.name.to_owned(),
            code: func.code_plaintext()?,
        })
    }

    /// Performs a "test" [`Func`] execution and returns the [result](TestExecuteFuncResult).
    #[instrument(name = "func.authoring.test_execute_func", level = "info", skip(ctx))]
    pub async fn test_execute_func(
        ctx: &DalContext,
        id: FuncId,
        args: serde_json::Value,
        execution_key: String,
        code: String,
        component_id: ComponentId,
    ) -> FuncAuthoringResult<TestExecuteFuncResult> {
        // Cache the old code.
        let func = Func::get_by_id_or_error(ctx, id).await?;
        let cached_code = func.code_base64.to_owned();

        // Use our new code and re-fetch.
        Func::modify_by_id(ctx, id, |func| {
            func.code_base64 = Some(general_purpose::STANDARD_NO_PAD.encode(code));
            Ok(())
        })
        .await?;
        let func_with_temp_code = Func::get_by_id_or_error(ctx, id).await?;

        // Perform the test execution.
        let test_execute_func_result = test_execute::perform_test_execution(
            ctx,
            func_with_temp_code,
            args,
            execution_key,
            component_id,
        )
        .await?;

        // Restore the old code. We need to do this in case users want to perform a commit.
        Func::modify_by_id(ctx, id, |func| {
            func.code_base64 = cached_code;
            Ok(())
        })
        .await?;

        Ok(test_execute_func_result)
    }

    /// Executes a [`Func`].
    #[instrument(name = "func.authoring.execute_func", level = "info", skip(ctx))]
    pub async fn execute_func(ctx: &DalContext, id: FuncId) -> FuncAuthoringResult<()> {
        let func = Func::get_by_id_or_error(ctx, id).await?;

        match func.kind {
            FuncKind::Qualification | FuncKind::CodeGeneration | FuncKind::Attribute => {
                execute::execute_attribute_func(ctx, &func).await?
            }
            FuncKind::Action => {
                // TODO(nick): fully restore or wait for actions v2. Essentially, we need to run
                // every prototype using the func id for every component.
                warn!("skipping action execution...");
                return Ok(());
            }
            kind => return Err(FuncAuthoringError::NotRunnable(id, kind)),
        };

        Ok(())
    }

    /// Saves a [`Func`].
    #[instrument(name = "func.authoring.save_func", level = "info", skip(ctx))]
    pub async fn save_func(
        ctx: &DalContext,
        id: FuncId,
        display_name: Option<String>,
        name: String,
        description: Option<String>,
        code: Option<String>,
        associations: Option<FuncAssociations>,
    ) -> FuncAuthoringResult<()> {
        let func = Func::get_by_id_or_error(ctx, id).await?;

        Func::modify_by_id(ctx, func.id, |func| {
            display_name.clone_into(&mut func.display_name);
            name.clone_into(&mut func.name);
            description.clone_into(&mut func.description);
            func.code_base64 = code
                .as_ref()
                .map(|code| general_purpose::STANDARD_NO_PAD.encode(code));

            Ok(())
        })
        .await?;

        if let Some(associations) = associations {
            let update_associations_start = tokio::time::Instant::now();
            save::update_associations(ctx, &func, associations).await?;
            debug!(%func.id, %func.kind,
                "updating associations took {:?}",
                update_associations_start.elapsed()
            );
        }

        Ok(())
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
        action_kind: ActionKind,
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
    /// The Function Run ID for the test
    pub func_run_id: FuncRunId,
}

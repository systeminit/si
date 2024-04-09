//! This module contains backend functionality for the [`Func`] authoring experience.

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

use thiserror::Error;

use crate::attribute::prototype::AttributePrototypeError;
use crate::func::argument::FuncArgumentError;
use crate::func::view::FuncViewError;
use crate::func::FuncKind;
use crate::{ActionPrototypeError, FuncError, SchemaVariantError};

mod create;
mod save;
mod types;

pub use create::create_func;
pub use create::AttributeOutputLocation;
pub use create::CreateFuncOptions;
pub use create::CreatedFunc;
pub use save::save_func;
pub use save::SavedFunc;
pub use types::compile_langjs_types;
pub use types::compile_return_types;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncAuthoringError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func named \"{0}\" already exists in this change set")]
    FuncNameExists(String),
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

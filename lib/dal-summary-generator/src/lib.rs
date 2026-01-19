//! This is a DAL-wrapper crate for functions used for solely "graph lab".

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
    // TODO(nick): enable this and add docs.
    // missing_docs,
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

use dal::{
    ComponentError,
    action::ActionError,
    attribute::value::AttributeValueError,
    diagram::DiagramError,
};
use si_layer_cache::LayerDbError;

mod component_summary;

pub use component_summary::*;

#[remain::sorted]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("diagram error: {0}")]
    Diagram(#[from] DiagramError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
}

//! This crate is an extension of the dal that handles materialized view generation.
//!
//! Why is it not in the dal? The dal is the kitchen sink for interwoven dependencies. Only
//! "edda-server" needs the MV generation bits and not "sdf-server", so having a crate that depends
//! on the dal allows the dal to be cached and does not affect the build times of sdf.
//!
//! The pattern for working in this crate is the following:
//! ```rust
//! # Create a module in the root (or nested)
//! pub mod your_mv_domain;
//!
//! # Example: single instance function using the entity ID
//! pub async fn as_frontend_type(_ctx: &DalContext, id: YourEntityId) -> super::Result<YourMV> {
//!     Ok(YourMv {
//!         id,
//!         name: "poop".to_string()
//!     })
//! }
//!
//! # Example: list function using the change set ID
//! pub async fn as_frontend_list_type(ctx: &DalContext) -> super::Result<YourMV> {
//!     Ok(YourMv {
//!         id: ctx.change_set_id(),
//!         things: Vec::new()
//!     })
//! }
//! ```

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

pub mod action_prototype_view_list;
pub mod action_view_list;
pub mod component;
pub mod component_list;
pub mod incoming_connections;
pub mod incoming_connections_list;
pub mod mgmt_prototype_view_list;
pub mod schema_variant;
pub mod schema_variant_categories;
pub mod secret;
pub mod view;
pub mod view_component_list;
pub mod view_list;

#[remain::sorted]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("action error: {0}")]
    Action(#[from] dal::action::ActionError),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] dal::action::prototype::ActionPrototypeError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] dal::attribute::prototype::AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(
        #[from] dal::attribute::prototype::argument::AttributePrototypeArgumentError,
    ),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] dal::attribute::value::AttributeValueError),
    #[error("cached module error: {0}")]
    CachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("dal transactions error: {0}")]
    DalTransactions(#[from] dal::TransactionsError),
    #[error("diagram error: {0}")]
    Diagram(#[from] dal::diagram::DiagramError),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("mgmt prototype error: {0}")]
    ManagementPrototype(#[from] dal::management::prototype::ManagementPrototypeError),
    #[error("prop error: {0}")]
    Prop(#[from] dal::prop::PropError),
    #[error("qualification summary error: {0}")]
    QualificationSummary(#[from] dal::qualification::QualificationSummaryError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("secret error: {0}")]
    Secret(#[from] dal::SecretError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(#[from] dal::validation::ValidationError),
}

type Result<T> = std::result::Result<T, Error>;

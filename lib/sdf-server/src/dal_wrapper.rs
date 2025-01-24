//! This module contains business logic wrapping functionality from the DAL that cannot (and should
//! not) be in the DAL itself.
//!
//! _Warning:_ this module should only be used as a last resort! Business logic should live in
//! other crates by default.

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

pub mod change_set_approval;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, thiserror::Error)]
pub enum DalWrapperError {
    #[error("approval requirement error: {0}")]
    ApprovalRequirement(#[from] dal::approval_requirement::ApprovalRequirementError),
    #[error("change set approval error")]
    ChangeSetApproval(#[from] dal::change_set::approval::ChangeSetApprovalError),
    #[error("invalid user found")]
    InvalidUser,
    #[error("missing applicable approval id")]
    MissingApplicableApproval(si_id::ChangeSetApprovalId),
    #[error("permissions error: {0}")]
    Permissions(#[from] permissions::Error),
    #[error("spicedb lookup subjects error: {0}")]
    SpiceDBLookupSubjects(#[source] si_data_spicedb::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
}

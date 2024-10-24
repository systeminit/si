use thiserror::Error;
use ulid::MonotonicError;

use crate::{ChangeSetError, ChangeSetId, TransactionsError, WorkspaceError, WorkspacePk};

mod fake_data_for_frontend;

pub use fake_data_for_frontend::filter_and_paginate;
pub use fake_data_for_frontend::generate;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AuditLogError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set not found: {0}")]
    ChangeSetNotFound(ChangeSetId),
    #[error("monotonic error: {0}")]
    Monotonic(#[from] MonotonicError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
}

pub type AuditLogResult<T> = Result<T, AuditLogError>;

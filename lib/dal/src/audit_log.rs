use chrono::Utc;
use si_events::audit_log::{AuditLog, AuditLogKind, AuditLogService};
use si_layer_cache::LayerDbError;
use thiserror::Error;
use ulid::MonotonicError;

use crate::layer_db_types::AuditLogContent;
use crate::{
    ChangeSetError, ChangeSetId, DalContext, TransactionsError, WorkspaceError, WorkspacePk,
};

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
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
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

pub fn assemble(content: AuditLogContent) -> AuditLog {
    match content {
        AuditLogContent::V1(inner) => AuditLog {
            actor: inner.actor,
            service: inner.service,
            kind: inner.kind,
            timestamp: inner.timestamp,
            origin_ip_address: inner.origin_ip_address,
            workspace_id: inner.workspace_id,
            change_set_id: inner.change_set_id,
        },
    }
}

pub fn new(
    ctx: &DalContext,
    service: AuditLogService,
    kind: AuditLogKind,
) -> AuditLogResult<AuditLog> {
    Ok(AuditLog {
        actor: ctx.events_actor(),
        service,
        kind,
        timestamp: Utc::now().to_rfc3339(),
        origin_ip_address: None,
        workspace_id: ctx.workspace_pk()?.into(),
        change_set_id: ctx.change_set_id().into(),
    })
}

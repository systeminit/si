use std::str::FromStr;

use audit_logs::database::{AuditDatabaseError, AuditLogRow};
use naxum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use si_data_nats::Subject;
use si_events::{audit_log::AuditLog, WorkspacePk};
use telemetry::prelude::*;
use thiserror::Error;

use super::app_state::AppState;

#[remain::sorted]
#[derive(Debug, Error)]
pub(crate) enum HandlerError {
    #[error("audit database error: {0}")]
    AuditDatabase(#[from] AuditDatabaseError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("unexpected subject shape: {0}")]
    UnexpectedSubjectShape(Subject),
}

type Result<T> = std::result::Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self, "failed to process message");
        Response::default_internal_server_error()
    }
}

pub(crate) async fn default(
    State(state): State<AppState>,
    subject: Subject,
    Json(audit_log): Json<AuditLog>,
) -> Result<()> {
    // Hitting an error when finding the workspace id should be impossible as we match the subject using middleware
    // before we get here.
    let workspace_id = find_workspace_id(subject, state.using_prefix())?;

    match audit_log {
        AuditLog::V1(inner) => {
            AuditLogRow::insert(
                state.context(),
                workspace_id,
                inner.kind,
                inner.timestamp,
                inner.change_set_id,
                inner.actor,
                Some(inner.entity_name),
            )
            .await?;
        }
    }
    Ok(())
}

// NOTE(nick,fletcher): we may be able to remove this if we store the workspace id on the audit log object itself, and
// we have a plan for old messages.
fn find_workspace_id(subject: Subject, using_prefix: bool) -> Result<WorkspacePk> {
    let mut parts = subject.split('.');
    if using_prefix {
        if let (Some(_prefix), Some(_p1), Some(_p2), Some(workspace_id)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        {
            Ok(WorkspacePk::from_str(workspace_id)?)
        } else {
            Err(HandlerError::UnexpectedSubjectShape(subject))
        }
    } else if let (Some(_p1), Some(_p2), Some(workspace_id)) =
        (parts.next(), parts.next(), parts.next())
    {
        Ok(WorkspacePk::from_str(workspace_id)?)
    } else {
        Err(HandlerError::UnexpectedSubjectShape(subject))
    }
}

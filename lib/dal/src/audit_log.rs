// TODO(nick): move this into its own crate.

use std::{collections::HashSet, str::FromStr};

use chrono::Utc;
use rand::{thread_rng, Rng};
use si_events::{audit_log::AuditLogKind, Actor, UserPk};
use thiserror::Error;
use ulid::MonotonicError;

use crate::{
    ChangeSet, ChangeSetError, ChangeSetId, DalContext, TransactionsError, Workspace,
    WorkspaceError, WorkspacePk,
};

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

/// Generate somewhat believable, but fake [`si_frontend_types::AuditLogs`](si_frontend_types::AuditLog).
pub async fn generate(
    ctx: &DalContext,
    generation_count: usize,
) -> AuditLogResult<Vec<si_frontend_types::AuditLog>> {
    let workspace_pk = ctx.workspace_pk()?;
    let workspace = Workspace::get_by_pk(ctx, &workspace_pk)
        .await?
        .ok_or(AuditLogError::WorkspaceNotFound(workspace_pk))?;

    let current_change_set_id = ctx.change_set_id();
    let current_change_set = ChangeSet::find(ctx, current_change_set_id)
        .await?
        .ok_or(AuditLogError::ChangeSetNotFound(current_change_set_id))?;

    let head_change_set_id = workspace.default_change_set_id();
    let head_change_set = ChangeSet::find(ctx, head_change_set_id)
        .await?
        .ok_or(AuditLogError::ChangeSetNotFound(current_change_set_id))?;

    let mut generator = ulid::Generator::new();
    let user_max = (generator.generate()?, "Max Verstappen", "max@siandrbr.dev");
    let user_charles = (
        generator.generate()?,
        "Charles LeClerc",
        "charles@ferrarisi.org",
    );
    let user_lewis = (
        generator.generate()?,
        "Lewis Hamilton",
        "lewis@mbamgpetronas+si.com",
    );

    let mut audit_logs = Vec::new();

    for _ in 0..generation_count {
        let (actor, actor_name, actor_email, origin_ip_address) = match dice_roll(2) {
            1 => match dice_roll(3) {
                1 => (
                    Actor::User(user_max.0.into()),
                    Some(user_max.1.to_owned()),
                    Some(user_max.2.to_owned()),
                    Some("127.0.0.1".to_string()),
                ),
                2 => (
                    Actor::User(user_charles.0.into()),
                    Some(user_charles.1.to_owned()),
                    Some(user_charles.2.to_owned()),
                    Some("127.0.0.1".to_string()),
                ),
                _ => (
                    Actor::User(user_lewis.0.into()),
                    Some(user_lewis.1.to_owned()),
                    Some(user_lewis.2.to_owned()),
                    Some("127.0.0.1".to_string()),
                ),
            },
            _ => (Actor::System, None, None, None),
        };

        let (kind, change_set_id, change_set_name) = match actor {
            Actor::User(_) => match dice_roll(2) {
                1 => (
                    AuditLogKind::CreateComponent,
                    current_change_set_id,
                    current_change_set.name.to_owned(),
                ),
                2 => (
                    AuditLogKind::DeleteComponent,
                    current_change_set_id,
                    current_change_set.name.to_owned(),
                ),
                _ => (
                    AuditLogKind::UpdatePropertyEditorValue,
                    current_change_set_id,
                    current_change_set.name.to_owned(),
                ),
            },
            Actor::System => match dice_roll(4) {
                1 => (
                    AuditLogKind::PerformedRebase,
                    head_change_set_id,
                    head_change_set.name.to_owned(),
                ),
                2 => (
                    AuditLogKind::PerformedRebase,
                    current_change_set_id,
                    current_change_set.name.to_owned(),
                ),
                3 => (
                    AuditLogKind::RanAction,
                    head_change_set_id,
                    head_change_set.name.to_owned(),
                ),
                _ => (
                    AuditLogKind::RanDependentValuesUpdate,
                    current_change_set_id,
                    current_change_set.name.to_owned(),
                ),
            },
        };

        audit_logs.push(si_frontend_types::AuditLog {
            actor,
            actor_name,
            actor_email,
            kind,
            timestamp: Utc::now().to_rfc3339(),
            origin_ip_address,
            workspace_id: workspace_pk.into(),
            workspace_name: Some(workspace.name().to_owned()),
            change_set_id: Some(change_set_id.to_string()),
            change_set_name: Some(change_set_name),
        });
    }

    Ok(audit_logs)
}

#[allow(clippy::too_many_arguments)]
pub fn filter_and_paginate(
    audit_logs: Vec<si_frontend_types::AuditLog>,
    page: Option<usize>,
    page_size: Option<usize>,
    sort_timestamp_ascending: Option<bool>,
    exclude_system_user: Option<bool>,
    kind_filter: HashSet<AuditLogKind>,
    change_set_filter: HashSet<ChangeSetId>,
    user_filter: HashSet<UserPk>,
) -> AuditLogResult<(Vec<si_frontend_types::AuditLog>, usize)> {
    // First, filter the logs based on our chosen filters. This logic works by processing each
    // audit log and assuming each log is within our desired scope by default. The instant that a
    // log does not meet our scope, we continue!
    let mut filtered_audit_logs = Vec::new();
    for audit_log in audit_logs {
        if !kind_filter.is_empty() && !kind_filter.contains(&audit_log.kind) {
            continue;
        }

        if let Some(change_set_id) = &audit_log.change_set_id {
            if !change_set_filter.is_empty()
                && !change_set_filter.contains(&ChangeSetId::from_str(change_set_id)?)
            {
                continue;
            }
        } else if !change_set_filter.is_empty() {
            continue;
        }

        match &audit_log.actor {
            Actor::User(user_pk) => {
                if !user_filter.is_empty() && !user_filter.contains(user_pk) {
                    continue;
                }
            }
            Actor::System => {
                if let Some(true) = exclude_system_user {
                    continue;
                }
            }
        }

        filtered_audit_logs.push(audit_log);
    }

    // After filtering, perform the sort.
    if let Some(true) = sort_timestamp_ascending {
        filtered_audit_logs.reverse();
    }

    // Count the number of audit logs after filtering, but before pagination. We need this so that
    // the frontend can know how many pages exists when paginating data.
    let total = filtered_audit_logs.len();

    // Finally, paginate and return.
    Ok((paginate(filtered_audit_logs, page, page_size), total))
}

fn paginate(
    logs: Vec<si_frontend_types::AuditLog>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Vec<si_frontend_types::AuditLog> {
    if let Some(page_size) = page_size {
        let target_page = page.unwrap_or(1);

        let mut current_page = 1;
        for chunk in logs.chunks(page_size) {
            if current_page == target_page {
                return chunk.to_vec();
            }
            current_page += 1;
        }
        logs
    } else {
        logs
    }
}

fn dice_roll(faces: usize) -> usize {
    thread_rng().gen_range(0..faces)
}

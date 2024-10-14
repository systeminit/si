// TODO(nick): move this into its own crate.

use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use si_events::{
    audit_log::{AuditLogKind, AuditLogService},
    Actor,
};
use thiserror::Error;
use ulid::MonotonicError;

use crate::{
    ChangeSet, ChangeSetError, ChangeSetId, DalContext, TransactionsError, Workspace,
    WorkspaceError, WorkspacePk,
};

const LOG_COUNT: usize = 25;

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
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
}

pub type AuditLogResult<T> = Result<T, AuditLogError>;

pub async fn generate(ctx: &DalContext) -> AuditLogResult<Vec<si_frontend_types::AuditLog>> {
    let workspace_pk = ctx.workspace_pk()?;
    let change_set_id = ctx.change_set_id();

    let workspace = Workspace::get_by_pk(ctx, &workspace_pk)
        .await?
        .ok_or(AuditLogError::WorkspaceNotFound(workspace_pk))?;
    let change_set = ChangeSet::find(ctx, change_set_id)
        .await?
        .ok_or(AuditLogError::ChangeSetNotFound(change_set_id))?;

    let mut generator = ulid::Generator::new();
    let mut audit_logs = Vec::new();

    for _ in 0..LOG_COUNT {
        let generated = thread_rng().gen_range(0..2);
        let (actor, actor_name, actor_email, origin_ip_address) = if generated == 1 {
            let rand_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(10)
                .map(char::from)
                .collect();
            let name = rand_string.to_uppercase();
            let email = format!("{rand_string}@poopcanoe.dev");
            let user_pk = generator.generate()?;
            (
                Actor::User(user_pk.into()),
                Some(name),
                Some(email),
                Some("127.0.0.1".to_string()),
            )
        } else {
            (Actor::System, None, None, None)
        };

        let (service, kind) = match actor {
            Actor::User(_) => {
                let generated = thread_rng().gen_range(0..3);
                if generated == 1 {
                    (AuditLogService::Sdf, AuditLogKind::CreateComponent)
                } else if generated == 2 {
                    (AuditLogService::Sdf, AuditLogKind::DeleteComponent)
                } else {
                    (
                        AuditLogService::Sdf,
                        AuditLogKind::UpdatePropertyEditorValue,
                    )
                }
            }
            Actor::System => {
                let generated = thread_rng().gen_range(0..2);
                if generated == 1 {
                    (AuditLogService::Rebaser, AuditLogKind::PerformedRebase)
                } else {
                    (
                        AuditLogService::Pinga,
                        AuditLogKind::RanDependentValuesUpdate,
                    )
                }
            }
        };

        audit_logs.push(si_frontend_types::AuditLog {
            actor,
            actor_name,
            actor_email,
            service,
            kind,
            timestamp: Utc::now().to_rfc3339(),
            origin_ip_address,
            workspace_id: workspace_pk.into(),
            workspace_name: Some(workspace.name().to_owned()),
            change_set_id: Some(change_set_id.to_string()),
            change_set_name: Some(change_set.name.to_owned()),
        });
    }

    Ok(audit_logs)
}

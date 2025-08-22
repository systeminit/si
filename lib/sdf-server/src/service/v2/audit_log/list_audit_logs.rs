use std::collections::HashMap;

use audit_database::AuditLogRow;
use axum::{
    Json,
    extract::{
        Path,
        Query,
        State,
    },
};
use dal::{
    ChangeSet,
    DalContext,
    audit_logging,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::User;
use si_events::{
    ChangeSetId,
    UserPk,
};
use si_frontend_types as frontend_types;

use super::AuditLogResult;
use crate::{
    AppState,
    extract::HandlerContext,
    service::v2::AccessBuilder,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsRequest {
    size: Option<usize>,
    sort_ascending: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsResponse {
    logs: Vec<frontend_types::AuditLog>,
    can_load_more: bool,
}

pub async fn list_audit_logs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id)): Path<(dal::WorkspacePk, dal::ChangeSetId)>,
    Query(request): Query<ListAuditLogsRequest>,
    State(state): State<AppState>,
) -> AuditLogResult<Json<ListAuditLogsResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let (database_logs, can_load_more) = audit_logging::list(
        &ctx,
        state.audit_database_context(),
        request.size.unwrap_or(0),
        request.sort_ascending.unwrap_or(false),
    )
    .await?;

    let mut assembler = Assembler::new();
    let mut logs = Vec::with_capacity(database_logs.len());
    for database_log in database_logs {
        logs.push(assembler.assemble(&ctx, database_log).await?);
    }

    Ok(Json(ListAuditLogsResponse {
        logs,
        can_load_more,
    }))
}

pub async fn list_audit_logs_for_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, component_id)): Path<(
        dal::WorkspacePk,
        dal::ChangeSetId,
        dal::ComponentId,
    )>,
    Query(request): Query<ListAuditLogsRequest>,
    State(state): State<AppState>,
) -> AuditLogResult<Json<ListAuditLogsResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let (database_logs, can_load_more) = audit_logging::list_for_component(
        &ctx,
        state.audit_database_context(),
        component_id,
        request.size.unwrap_or(0),
        request.sort_ascending.unwrap_or(false),
    )
    .await?;

    let mut assembler = Assembler::new();
    let mut logs = Vec::with_capacity(database_logs.len());
    for database_log in database_logs {
        logs.push(assembler.assemble(&ctx, database_log).await?);
    }

    Ok(Json(ListAuditLogsResponse {
        logs,
        can_load_more,
    }))
}

#[derive(Debug)]
struct Assembler {
    change_set_cache: HashMap<ChangeSetId, ChangeSet>,
    user_cache: HashMap<UserPk, User>,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            change_set_cache: HashMap::new(),
            user_cache: HashMap::new(),
        }
    }

    pub async fn assemble(
        &mut self,
        ctx: &DalContext,
        audit_log: AuditLogRow,
    ) -> AuditLogResult<si_frontend_types::AuditLog> {
        let (change_set_id, change_set_name) = self
            .find_change_set_metadata(ctx, audit_log.change_set_id)
            .await?;
        let (user_id, user_email, user_name) =
            self.find_user_metadata(ctx, audit_log.user_id).await?;

        Ok(si_frontend_types::AuditLog {
            title: audit_log.title,
            user_id,
            user_email,
            user_name,
            kind: audit_log.kind,
            // TODO(nick): allow this to be optional in the frontend.
            entity_name: audit_log.entity_name.unwrap_or(" ".to_string()),
            // NOTE(nick): this maintains compatibility from when these used to have whitespace-based names.
            // However, we should make this optional in the frontend.
            entity_type: audit_log.entity_type.unwrap_or(" ".to_string()),
            // NOTE(nick): this is specifically converted to ISO RFC 3339 for the frontend.
            timestamp: audit_log.timestamp.to_rfc3339(),
            change_set_id,
            change_set_name,
            // TODO(nick): allow this to be optional in the frontend.
            metadata: audit_log.metadata.unwrap_or(serde_json::Value::Null),
            authentication_method: audit_log.authentication_method,
        })
    }

    async fn find_change_set_metadata(
        &mut self,
        ctx: &DalContext,
        change_set_id: Option<ChangeSetId>,
    ) -> AuditLogResult<(Option<ChangeSetId>, Option<String>)> {
        match change_set_id {
            Some(change_set_id) => {
                let change_set_name =
                    if let Some(change_set) = self.change_set_cache.get(&change_set_id) {
                        change_set.name.to_owned()
                    } else {
                        let change_set = ChangeSet::get_by_id(ctx, change_set_id).await?;
                        let found_data = change_set.name.to_owned();
                        self.change_set_cache.insert(change_set_id, change_set);
                        found_data
                    };

                Ok((Some(change_set_id), Some(change_set_name)))
            }
            None => Ok((None, None)),
        }
    }

    async fn find_user_metadata(
        &mut self,
        ctx: &DalContext,
        user_id: Option<UserPk>,
    ) -> AuditLogResult<(Option<UserPk>, Option<String>, Option<String>)> {
        match user_id {
            None => Ok((None, None, None)),
            Some(user_id) => {
                if let Some(user) = self.user_cache.get(&user_id) {
                    Ok((
                        Some(user_id),
                        Some(user.email().to_owned()),
                        Some(user.name().to_owned()),
                    ))
                } else {
                    let user = User::get_by_pk(ctx, user_id).await?;
                    let found_data = (
                        Some(user_id),
                        Some(user.email().to_owned()),
                        Some(user.name().to_owned()),
                    );
                    self.user_cache.insert(user_id, user);
                    Ok(found_data)
                }
            }
        }
    }
}

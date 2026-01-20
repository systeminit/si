use si_events::{
    ActionId,
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    FuncId,
    FuncRun,
    FuncRunId,
    WorkspacePk,
};

use crate::{
    SiDbError,
    SiDbContext,
    SiDbResult,
    transactions::SiDbTransactions as _,
};

pub const DBNAME: &str = "func_runs";

const READY_MANY_FOR_WORKSPACE_ID_QUERY: &str = "SELECT * FROM func_runs WHERE workspace_id = $1";

const GET_LAST_QUALIFICATION_FOR_ATTRIBUTE_VALUE_ID_QUERY: &str = "SELECT value FROM func_runs
        WHERE attribute_value_id = $2 AND workspace_id = $1
        ORDER BY updated_at DESC
        LIMIT 1";

const LIST_ACTION_HISTORY_QUERY: &str = "SELECT value FROM func_runs
        WHERE function_kind = 'Action' AND workspace_id = $1
        ORDER BY updated_at DESC";

const GET_LAST_ACTION_BY_ACTION_ID_QUERY: &str = "SELECT value FROM func_runs
        WHERE function_kind = 'Action' AND workspace_id = $1 AND action_id = $2
        ORDER BY updated_at DESC
        LIMIT 1";

const LIST_MANAGEMENT_HISTORY_QUERY: &str = "SELECT value FROM func_runs
        WHERE function_kind = 'Management' AND workspace_id = $1 AND change_set_id = $2 AND action_id IS NOT NULL
        ORDER BY updated_at DESC";

const GET_LAST_MANAGEMENT_BY_FUNC_AND_COMPONENT_ID_QUERY: &str = "SELECT value FROM func_runs
        WHERE function_kind = 'Management' AND workspace_id = $1 AND change_set_id = $2 AND component_id = $3 AND action_id = $4
        ORDER BY updated_at DESC
        LIMIT 1";

const PAGINATED_WORKSPACE_QUERY_WITH_CURSOR: &str = "SELECT * FROM func_runs
        WHERE workspace_id = $1
        AND change_set_id = $2
        AND (
            created_at < (SELECT created_at FROM func_runs WHERE key = $3) OR
            (created_at = (SELECT created_at FROM func_runs WHERE key = $3) AND key < $3)
        )
        ORDER BY created_at DESC, key DESC
        LIMIT $4";

const PAGINATED_WORKSPACE_QUERY_NO_CURSOR: &str = "SELECT * FROM func_runs
        WHERE workspace_id = $1
            AND change_set_id = $2
        ORDER BY created_at DESC, key DESC
        LIMIT $3";

const PAGINATED_COMPONENT_QUERY_WITH_CURSOR: &str = "SELECT * FROM func_runs
        WHERE workspace_id = $1
            AND change_set_id = $2
            AND component_id = $3
            AND (
                created_at < (SELECT created_at FROM func_runs WHERE key = $4) OR
                (created_at = (SELECT created_at FROM func_runs WHERE key = $4) AND key < $4)
            )
        ORDER BY created_at DESC, key DESC
        LIMIT $5";

const PAGINATED_COMPONENT_QUERY_NO_CURSOR: &str = "SELECT * FROM func_runs
        WHERE workspace_id = $1
            AND change_set_id = $2
            AND component_id = $3
        ORDER BY created_at DESC, key DESC
        LIMIT $4";

#[derive(Debug, Clone)]
pub struct FuncRunDb {
}

impl FuncRunDb {
    /// Write a new func run to the database.
    /// This function can be used to replace the layer-cache write() function.
    pub async fn upsert(
        ctx: &impl SiDbContext,
        func_run: FuncRun,
    ) -> SiDbResult<()> {
        let json: serde_json::Value = serde_json::to_value(&func_run)?;
        let postcard_bytes = postcard::to_stdvec(&func_run)
            .map_err(|e| SiDbError::Postcard(e.to_string()))?;

        ctx.txns()
            .await?
            .pg()
            .execute(
                "INSERT INTO func_runs (
                    key,
                    sort_key,
                    created_at,
                    updated_at,
                    state,
                    function_kind,
                    workspace_id,
                    change_set_id,
                    actor_id,
                    component_id,
                    attribute_value_id,
                    action_id,
                    action_originating_change_set_id,
                    json_value,
                    value
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9,
                    $10,
                    $11,
                    $12,
                    $13,
                    $14,
                    $15
                ) ON CONFLICT (key) DO UPDATE SET
                    updated_at = EXCLUDED.updated_at,
                    state = EXCLUDED.state,
                    json_value = EXCLUDED.json_value,
                    value = EXCLUDED.value",
                &[
                    &func_run.id().to_string(),
                    &func_run.tenancy().workspace_pk.to_string(),
                    &func_run.created_at(),
                    &func_run.updated_at(),
                    &func_run.state().to_string(),
                    &func_run.function_kind().to_string(),
                    &func_run.tenancy().workspace_pk.to_string(),
                    &func_run.tenancy().change_set_id.to_string(),
                    &func_run.actor().to_string(),
                    &func_run.component_id().map(|v| v.to_string()),
                    &func_run.attribute_value_id().map(|v| v.to_string()),
                    &func_run.action_id().map(|v| v.to_string()),
                    &func_run
                        .action_originating_change_set_id()
                        .map(|v| v.to_string()),
                    &json,
                    &postcard_bytes.as_slice(),
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn get_last_run_for_action_id_opt(
        ctx: &impl SiDbContext,
        workspace_pk: WorkspacePk,
        action_id: ActionId,
    ) -> SiDbResult<Option<FuncRun>> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                GET_LAST_ACTION_BY_ACTION_ID_QUERY,
                &[&workspace_pk, &action_id],
            )
            .await?;

        if let Some(row) = maybe_row {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run: FuncRun = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            Ok(Some(func_run))
        } else {
            Ok(None)
        }
    }

    pub async fn get_last_run_for_action_id(
        ctx: &impl SiDbContext,
        workspace_pk: WorkspacePk,
        action_id: ActionId,
    ) -> SiDbResult<FuncRun> {
        Self::get_last_run_for_action_id_opt(ctx, workspace_pk, action_id)
            .await?
            .ok_or(SiDbError::ActionIdNotFound(action_id))
    }

    pub async fn list_management_history(
        ctx: &impl SiDbContext,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> SiDbResult<Vec<FuncRun>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_MANAGEMENT_HISTORY_QUERY,
                &[&workspace_pk, &change_set_id],
            )
            .await?;

        let mut func_runs = Vec::with_capacity(rows.len());
        for row in rows {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run: FuncRun = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            func_runs.push(func_run);
        }

        Ok(func_runs)
    }

    pub async fn get_last_management_run_for_func_and_component_id(
        ctx: &impl SiDbContext,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        component_id: ComponentId,
        func_id: FuncId,
    ) -> SiDbResult<Option<FuncRun>> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                GET_LAST_MANAGEMENT_BY_FUNC_AND_COMPONENT_ID_QUERY,
                &[&workspace_pk, &change_set_id, &component_id, &func_id],
            )
            .await?;

        if let Some(row) = maybe_row {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run: FuncRun = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            Ok(Some(func_run))
        } else {
            Ok(None)
        }
    }

    pub async fn get_last_qualification_for_attribute_value_id(
        ctx: &impl SiDbContext,
        workspace_pk: WorkspacePk,
        attribute_value_id: AttributeValueId,
    ) -> SiDbResult<Option<FuncRun>> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                GET_LAST_QUALIFICATION_FOR_ATTRIBUTE_VALUE_ID_QUERY,
                &[&workspace_pk, &attribute_value_id],
            )
            .await?;

        if let Some(row) = maybe_row {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run: FuncRun = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            Ok(Some(func_run))
        } else {
            Ok(None)
        }
    }

    pub async fn read(ctx: &impl SiDbContext, key: FuncRunId) -> SiDbResult<Option<FuncRun>> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                &format!("SELECT value FROM {} WHERE key = $1", DBNAME),
                &[&key.to_string()],
            )
            .await?;

        if let Some(row) = maybe_row {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run: FuncRun = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            Ok(Some(func_run))
        } else {
            Ok(None)
        }
    }

    pub async fn try_read(ctx: &impl SiDbContext, key: FuncRunId) -> SiDbResult<FuncRun> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                &format!("SELECT value FROM {} WHERE key = $1", DBNAME),
                &[&key.to_string()],
            )
            .await?;

        if let Some(row) = maybe_row {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run: FuncRun = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            Ok(func_run)
        } else {
            Err(SiDbError::MissingFuncRun(key))
        }
    }

    pub async fn read_many_for_workspace(
        ctx: &impl SiDbContext,
        workspace_pk: WorkspacePk,
    ) -> SiDbResult<Vec<FuncRun>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(READY_MANY_FOR_WORKSPACE_ID_QUERY, &[&workspace_pk])
            .await?;

        let mut func_runs = Vec::with_capacity(rows.len());
        for row in rows {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run: FuncRun = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            func_runs.push(func_run);
        }

        Ok(func_runs)
    }

    /// Read function runs for a workspace with pagination support.
    ///
    /// This method uses cursor-based pagination where:
    /// - `limit` controls how many items to return per page
    /// - `cursor` is the ID of the last item from the previous page
    /// - Results are filtered by workspace_id and change_set_id
    ///
    /// Results are ordered by creation time (newest first).
    pub async fn read_many_for_workspace_paginated(
        ctx: &impl SiDbContext,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        limit: i64,
        cursor: Option<FuncRunId>,
    ) -> SiDbResult<Vec<FuncRun>> {
        let rows = if let Some(cursor_id) = cursor {
            ctx.txns()
                .await?
                .pg()
                .query(
                    PAGINATED_WORKSPACE_QUERY_WITH_CURSOR,
                    &[
                        &workspace_pk,
                        &change_set_id.to_string(),
                        &cursor_id.to_string(),
                        &limit,
                    ],
                )
                .await?
        } else {
            ctx.txns()
                .await?
                .pg()
                .query(
                    PAGINATED_WORKSPACE_QUERY_NO_CURSOR,
                    &[&workspace_pk, &change_set_id.to_string(), &limit],
                )
                .await?
        };

        let mut func_runs = Vec::with_capacity(rows.len());
        for row in rows {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run: FuncRun = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            func_runs.push(func_run);
        }

        Ok(func_runs)
    }

    /// Read function runs for a specific component with pagination support.
    ///
    /// This method uses cursor-based pagination where:
    /// - `limit` controls how many items to return per page
    /// - `cursor` is the ID of the last item from the previous page
    /// - Results are filtered by workspace_id, change_set_id, and component_id
    ///
    /// Results are ordered by creation time (newest first).
    pub async fn read_many_for_component_paginated(
        ctx: &impl SiDbContext,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        component_id: ComponentId,
        limit: i64,
        cursor: Option<FuncRunId>,
    ) -> SiDbResult<Vec<FuncRun>> {
        let rows = if let Some(cursor_id) = cursor {
            ctx.txns()
                .await?
                .pg()
                .query(
                    PAGINATED_COMPONENT_QUERY_WITH_CURSOR,
                    &[
                        &workspace_pk,
                        &change_set_id.to_string(),
                        &component_id.to_string(),
                        &cursor_id.to_string(),
                        &limit,
                    ],
                )
                .await?
        } else {
            ctx.txns()
                .await?
                .pg()
                .query(
                    PAGINATED_COMPONENT_QUERY_NO_CURSOR,
                    &[
                        &workspace_pk,
                        &change_set_id.to_string(),
                        &component_id.to_string(),
                        &limit,
                    ],
                )
                .await?
        };

        let mut func_runs = Vec::with_capacity(rows.len());
        for row in rows {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run: FuncRun = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            func_runs.push(func_run);
        }

        Ok(func_runs)
    }
}


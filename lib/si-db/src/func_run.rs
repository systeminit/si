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
    SiDbContext,
    SiDbError,
    SiDbResult,
    transactions::SiDbTransactions as _,
};

pub const DBNAME: &str = "func_runs";

const READY_MANY_FOR_WORKSPACE_ID_QUERY: &str = "SELECT * FROM func_runs WHERE workspace_id = $1";

const GET_LAST_QUALIFICATION_FOR_ATTRIBUTE_VALUE_ID_QUERY: &str = "SELECT value FROM func_runs
        WHERE attribute_value_id = $2 AND workspace_id = $1
        ORDER BY updated_at DESC
        LIMIT 1";

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
pub struct FuncRunDb {}

impl FuncRunDb {
    /// Write a new func run to the database.
    /// This function can be used to replace the layer-cache write() function.
    pub async fn upsert(ctx: &impl SiDbContext, func_run: FuncRun) -> SiDbResult<()> {
        let json: serde_json::Value = serde_json::to_value(&func_run)?;
        let postcard_bytes =
            postcard::to_stdvec(&func_run).map_err(|e| SiDbError::Postcard(e.to_string()))?;

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

    /// Write multiple func runs to the database in a single INSERT query.
    /// This is more efficient than calling upsert multiple times.
    pub async fn upsert_batch(ctx: &impl SiDbContext, func_runs: Vec<FuncRun>) -> SiDbResult<()> {
        if func_runs.is_empty() {
            return Ok(());
        }

        // Build the VALUES part of the query dynamically
        // Store all the data we need to keep alive for the query
        struct RowData {
            id: String,
            workspace_pk: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            state: String,
            function_kind: String,
            change_set_id: String,
            actor: String,
            component_id: Option<String>,
            attribute_value_id: Option<String>,
            action_id: Option<String>,
            action_originating_change_set_id: Option<String>,
            json: serde_json::Value,
            postcard_bytes: Vec<u8>,
        }

        let mut values_clauses = Vec::new();
        let mut row_data_vec = Vec::new();
        let mut param_index = 1;

        for func_run in &func_runs {
            let json: serde_json::Value = serde_json::to_value(func_run)?;
            let postcard_bytes =
                postcard::to_stdvec(func_run).map_err(|e| SiDbError::Postcard(e.to_string()))?;

            // Create placeholders for this row ($1, $2, ... $15)
            let placeholders: Vec<String> = (param_index..param_index + 15)
                .map(|i| format!("${i}"))
                .collect();
            values_clauses.push(format!("({})", placeholders.join(", ")));

            row_data_vec.push(RowData {
                id: func_run.id().to_string(),
                workspace_pk: func_run.tenancy().workspace_pk.to_string(),
                created_at: func_run.created_at(),
                updated_at: func_run.updated_at(),
                state: func_run.state().to_string(),
                function_kind: func_run.function_kind().to_string(),
                change_set_id: func_run.tenancy().change_set_id.to_string(),
                actor: func_run.actor().to_string(),
                component_id: func_run.component_id().map(|v| v.to_string()),
                attribute_value_id: func_run.attribute_value_id().map(|v| v.to_string()),
                action_id: func_run.action_id().map(|v| v.to_string()),
                action_originating_change_set_id: func_run
                    .action_originating_change_set_id()
                    .map(|v| v.to_string()),
                json,
                postcard_bytes,
            });

            param_index += 15;
        }

        let query = format!(
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
            ) VALUES {}
            ON CONFLICT (key) DO UPDATE SET
                updated_at = EXCLUDED.updated_at,
                state = EXCLUDED.state,
                json_value = EXCLUDED.json_value,
                value = EXCLUDED.value",
            values_clauses.join(", ")
        );

        // Build the parameter array dynamically
        // We need to store the slices separately to keep them alive
        let postcard_slices: Vec<&[u8]> = row_data_vec
            .iter()
            .map(|rd| rd.postcard_bytes.as_slice())
            .collect();

        let mut params: Vec<&(dyn postgres_types::ToSql + Sync)> = Vec::new();
        for (idx, row_data) in row_data_vec.iter().enumerate() {
            params.push(&row_data.id);
            params.push(&row_data.workspace_pk);
            params.push(&row_data.created_at);
            params.push(&row_data.updated_at);
            params.push(&row_data.state);
            params.push(&row_data.function_kind);
            params.push(&row_data.workspace_pk);
            params.push(&row_data.change_set_id);
            params.push(&row_data.actor);
            params.push(&row_data.component_id);
            params.push(&row_data.attribute_value_id);
            params.push(&row_data.action_id);
            params.push(&row_data.action_originating_change_set_id);
            params.push(&row_data.json);
            params.push(&postcard_slices[idx]);
        }

        ctx.txns().await?.pg().execute(&query, &params[..]).await?;

        Ok(())
    }

    /// Returns the IDs from the input batch that do NOT exist in the database.
    /// This is useful for determining which func runs need to be migrated.
    pub async fn find_missing_ids(
        ctx: &impl SiDbContext,
        ids: &[FuncRunId],
    ) -> SiDbResult<Vec<FuncRunId>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Convert IDs to strings for the query
        let id_strings: Vec<String> = ids.iter().map(|id| id.to_string()).collect();

        // Build a query with ANY to check which IDs exist
        let query = format!("SELECT key FROM {DBNAME} WHERE key = ANY($1)");

        let rows = ctx.txns().await?.pg().query(&query, &[&id_strings]).await?;

        // Collect the IDs that exist in the database
        let existing_ids: std::collections::HashSet<String> =
            rows.iter().map(|row| row.get::<_, String>("key")).collect();

        // Return the IDs that don't exist
        let missing_ids: Vec<FuncRunId> = ids
            .iter()
            .filter(|id| !existing_ids.contains(&id.to_string()))
            .copied()
            .collect();

        Ok(missing_ids)
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
                &format!("SELECT value FROM {DBNAME} WHERE key = $1"),
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
                &format!("SELECT value FROM {DBNAME} WHERE key = $1"),
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

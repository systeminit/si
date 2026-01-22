use std::sync::Arc;

use si_events::{
    FuncRunId,
    FuncRunLog,
    FuncRunLogId,
};

use crate::{
    SiDbContext,
    SiDbError,
    SiDbResult,
    transactions::SiDbTransactions as _,
};

pub const DBNAME: &str = "func_run_logs";

#[derive(Debug, Clone)]
pub struct FuncRunLogDb {}

impl FuncRunLogDb {
    /// Write a new func run log to the database.
    /// This function writes to both si-db and layer-db for backward compatibility during migration.
    pub async fn upsert(ctx: &impl SiDbContext, func_run_log: FuncRunLog) -> SiDbResult<()> {
        let postcard_bytes =
            postcard::to_stdvec(&func_run_log).map_err(|e| SiDbError::Postcard(e.to_string()))?;

        // Write to si-db
        ctx.txns()
            .await?
            .pg()
            .execute(
                &format!(
                    "INSERT INTO {DBNAME} (
                    key,
                    created_at,
                    updated_at,
                    workspace_id,
                    change_set_id,
                    func_run_id,
                    value
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7
                ) ON CONFLICT (key) DO UPDATE SET
                    updated_at = EXCLUDED.updated_at,
                    value = EXCLUDED.value;"
                ),
                &[
                    &func_run_log.id().to_string(),
                    &func_run_log.created_at(),
                    &func_run_log.updated_at(),
                    &func_run_log.tenancy().workspace_pk.to_string(),
                    &func_run_log.tenancy().change_set_id.to_string(),
                    &func_run_log.func_run_id().to_string(),
                    &postcard_bytes.as_slice(),
                ],
            )
            .await?;

        // Also write to layer-db for backward compatibility during migration
        // Convert HistoryActor to Actor
        let actor = match ctx.history_actor() {
            crate::history_event::HistoryActor::SystemInit => si_events::Actor::System,
            crate::history_event::HistoryActor::User(pk) => si_events::Actor::User(*pk),
        };

        ctx.func_run_log_layer_db()
            .write(
                Arc::new(func_run_log.clone()),
                None, // web_events
                func_run_log.tenancy(),
                actor,
            )
            .await
            .map_err(|e| SiDbError::LayerDb(e.to_string()))?;

        Ok(())
    }

    pub async fn get_for_func_run_id(
        ctx: &impl SiDbContext,
        func_run_id: FuncRunId,
    ) -> SiDbResult<Option<FuncRunLog>> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                &format!("SELECT value FROM {DBNAME} WHERE func_run_id = $1"),
                &[&func_run_id],
            )
            .await?;

        if let Some(row) = maybe_row {
            let value_bytes: Vec<u8> = row.try_get("value")?;
            let func_run_log: FuncRunLog = postcard::from_bytes(&value_bytes)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;
            Ok(Some(func_run_log))
        } else {
            // Fall back to layer-db if not found in si-db
            ctx.func_run_log_layer_db()
                .get_for_func_run_id(func_run_id)
                .await
                .map(|opt_arc| opt_arc.map(|arc| (*arc).clone()))
                .map_err(|e| SiDbError::LayerDb(e.to_string()))
        }
    }

    /// Returns the IDs from the input batch that do NOT exist in the database.
    /// This is useful for determining which func run logs need to be migrated.
    pub async fn find_missing_ids(
        ctx: &impl SiDbContext,
        ids: &[FuncRunLogId],
    ) -> SiDbResult<Vec<FuncRunLogId>> {
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
        let missing_ids: Vec<FuncRunLogId> = ids
            .iter()
            .filter(|id| !existing_ids.contains(&id.to_string()))
            .copied()
            .collect();

        Ok(missing_ids)
    }

    /// Write multiple func run logs to the database in a single INSERT query.
    /// This is more efficient than calling upsert multiple times.
    pub async fn upsert_batch(
        ctx: &impl SiDbContext,
        func_run_logs: Vec<FuncRunLog>,
    ) -> SiDbResult<()> {
        if func_run_logs.is_empty() {
            return Ok(());
        }

        // Store all the data we need to keep alive for the query
        struct RowData {
            id: String,
            workspace_pk: String,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            change_set_id: String,
            func_run_id: String,
            postcard_bytes: Vec<u8>,
        }

        let mut values_clauses = Vec::new();
        let mut row_data_vec = Vec::new();
        let mut param_index = 1;

        const COL_COUNT: usize = 7;

        for func_run_log in &func_run_logs {
            let postcard_bytes = postcard::to_stdvec(func_run_log)
                .map_err(|e| SiDbError::Postcard(e.to_string()))?;

            // Create placeholders for this row ($1, $2, ... $COL_COUNT)
            let placeholders: Vec<String> = (param_index..param_index + COL_COUNT)
                .map(|i| format!("${i}"))
                .collect();
            values_clauses.push(format!("({})", placeholders.join(", ")));

            row_data_vec.push(RowData {
                id: func_run_log.id().to_string(),
                workspace_pk: func_run_log.tenancy().workspace_pk.to_string(),
                created_at: func_run_log.created_at(),
                updated_at: func_run_log.updated_at(),
                change_set_id: func_run_log.tenancy().change_set_id.to_string(),
                func_run_id: func_run_log.func_run_id().to_string(),
                postcard_bytes,
            });

            param_index += COL_COUNT;
        }

        let query = format!(
            "INSERT INTO {} (
                key,
                created_at,
                updated_at,
                workspace_id,
                change_set_id,
                func_run_id,
                value
            ) VALUES {}
            ON CONFLICT (key) DO UPDATE SET
                updated_at = EXCLUDED.updated_at,
                value = EXCLUDED.value",
            DBNAME,
            values_clauses.join(", ")
        );

        // Build the parameter array dynamically

        // This looks like extra work, but since the pg library expects refs of everything,
        // we had to create the row_data_vec to own the values while we pass them down
        let postcard_slices: Vec<&[u8]> = row_data_vec
            .iter()
            .map(|rd| rd.postcard_bytes.as_slice())
            .collect();

        let mut params: Vec<&(dyn postgres_types::ToSql + Sync)> = Vec::new();
        for (idx, row_data) in row_data_vec.iter().enumerate() {
            params.push(&row_data.id);
            params.push(&row_data.created_at);
            params.push(&row_data.updated_at);
            params.push(&row_data.workspace_pk);
            params.push(&row_data.change_set_id);
            params.push(&row_data.func_run_id);
            params.push(&postcard_slices[idx]);
        }

        ctx.txns().await?.pg().execute(&query, &params[..]).await?;

        Ok(())
    }
}

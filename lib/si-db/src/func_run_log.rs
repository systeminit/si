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
    /// This function can be used to replace the layer-cache write() function.
    pub async fn upsert(
        ctx: &impl SiDbContext,
        func_run_log: FuncRunLog,
    ) -> SiDbResult<()> {
        let postcard_bytes = postcard::to_stdvec(&func_run_log)
            .map_err(|e| SiDbError::Postcard(e.to_string()))?;

        ctx.txns()
            .await?
            .pg()
            .execute(
                &format!(
                    "INSERT INTO {DBNAME} (
                    key,
                    sort_key,
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
                    $7,
                    $8
                ) ON CONFLICT (key) DO UPDATE SET
                    updated_at = EXCLUDED.updated_at,
                    value = EXCLUDED.value;"
                ),
                &[
                    &func_run_log.id().to_string(),
                    &func_run_log.tenancy().workspace_pk.to_string(),
                    &func_run_log.created_at(),
                    &func_run_log.updated_at(),
                    &func_run_log.tenancy().workspace_pk.to_string(),
                    &func_run_log.tenancy().change_set_id.to_string(),
                    &func_run_log.func_run_id().to_string(),
                    &postcard_bytes.as_slice(),
                ],
            )
            .await?;

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
            Ok(None)
        }
    }
}

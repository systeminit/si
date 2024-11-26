use crate::{DalContext, WsEvent, WsEventResult, WsPayload};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use telemetry::prelude::*;
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PromptOverrideError {
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("transactions error: {0}")]
    Transactions(#[from] crate::TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] crate::WsEventError),
}

pub type Result<T> = std::result::Result<T, PromptOverrideError>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptUpdatedPayload {
    pub kind: String,
    pub overridden: bool,
}

impl WsEvent {
    pub async fn prompt_updated(
        ctx: &DalContext,
        kind: String,
        overridden: bool,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::PromptUpdated(PromptUpdatedPayload { kind, overridden }),
        )
        .await
    }
}

pub struct PromptOverride;

impl PromptOverride {
    pub async fn list(ctx: &DalContext) -> Result<HashSet<String>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "
                    SELECT kind FROM prompt_overrides
                ",
                &[],
            )
            .await?;
        let mut result = HashSet::with_capacity(rows.len());
        for row in rows {
            result.insert(row.try_get(0)?);
        }
        Ok(result)
    }

    pub async fn get_opt(ctx: &DalContext, kind: &str) -> Result<Option<String>> {
        match ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "
                    SELECT prompt_yaml FROM prompt_overrides WHERE kind = $1
                ",
                &[&kind],
            )
            .await?
        {
            Some(row) => Ok(Some(row.try_get(0)?)),
            None => Ok(None),
        }
    }

    pub async fn set(ctx: &DalContext, kind: &str, prompt: &str) -> Result<()> {
        ctx.txns()
            .await?
            .pg()
            .execute(
                "
                    INSERT INTO prompt_overrides
                        (kind, prompt_yaml)
                        VALUES
                        ($1, $2)
                    ON CONFLICT (kind) DO
                    UPDATE SET prompt_yaml = $2
                ",
                &[&kind, &prompt],
            )
            .await?;

        WsEvent::prompt_updated(ctx, kind.to_owned(), true)
            .await?
            .publish_immediately(ctx)
            .await?;
        Ok(())
    }

    pub async fn reset(ctx: &DalContext, kind: &str) -> Result<bool> {
        let deleted = ctx
            .txns()
            .await?
            .pg()
            .execute(
                "
                DELETE FROM prompt_overrides WHERE kind = $1
            ",
                &[&kind],
            )
            .await?;
        if deleted > 0 {
            WsEvent::prompt_updated(ctx, kind.to_owned(), false)
                .await?
                .publish_immediately(ctx)
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

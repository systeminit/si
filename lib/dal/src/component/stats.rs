//! This module contains [`ComponentStats`].

use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use telemetry::prelude::*;
use tokio_postgres::row::RowIndex;
use tokio_postgres::Row;

use crate::component::ComponentResult;
use crate::{ComponentId, DalContext, StandardModelResult};

const LIST_MODIFIED: &str = include_str!("../queries/component_stats_list_modified.sql");
const LIST_ADDED: &str = include_str!("../queries/component_stats_list_added.sql");
const LIST_DELETED: &str = include_str!("../queries/component_stats_list_deleted.sql");

/// A collection of statistics for [`Components`](crate::Component) in the current
/// [`ChangeSet`](crate::ChangeSet).
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ComponentStats {
    added: usize,
    deleted: usize,
    modified: usize,
}

impl ComponentStats {
    pub async fn new(ctx: &DalContext<'_, '_>) -> ComponentResult<Self> {
        let component_stats = if ctx.visibility().is_head() {
            Self::default()
        } else {
            let added = Self::list_added(ctx).await?.len();
            let deleted = Self::list_deleted(ctx).await?.len();
            let modified = Self::list_modified(ctx).await?.len();
            Self {
                added,
                deleted,
                modified,
            }
        };
        Ok(component_stats)
    }

    #[instrument(skip_all)]
    async fn list_added(ctx: &DalContext<'_, '_>) -> ComponentResult<Vec<ComponentId>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_ADDED,
                &[
                    ctx.read_tenancy(),
                    &ctx.visibility().change_set_pk,
                    &ctx.visibility().edit_session_pk,
                ],
            )
            .await?;
        Ok(Self::component_ids_from_rows_with_idx(rows, "id")?)
    }

    #[instrument(skip_all)]
    async fn list_deleted(ctx: &DalContext<'_, '_>) -> ComponentResult<Vec<ComponentId>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_DELETED,
                &[
                    ctx.read_tenancy(),
                    &ctx.visibility().change_set_pk,
                    &ctx.visibility().edit_session_pk,
                ],
            )
            .await?;
        Ok(Self::component_ids_from_rows_with_idx(rows, "id")?)
    }

    #[instrument(skip_all)]
    async fn list_modified(ctx: &DalContext<'_, '_>) -> ComponentResult<Vec<ComponentId>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_MODIFIED,
                &[
                    ctx.read_tenancy(),
                    &ctx.visibility().change_set_pk,
                    &ctx.visibility().edit_session_pk,
                ],
            )
            .await?;
        Ok(Self::component_ids_from_rows_with_idx(
            rows,
            "attribute_context_component_id",
        )?)
    }

    /// Modification of [`standard_model::objects_from_rows()`] for resolving rows to
    /// [`ComponentIds`](crate::Component).
    fn component_ids_from_rows_with_idx<I>(
        rows: Vec<Row>,
        idx: I,
    ) -> StandardModelResult<Vec<ComponentId>>
    where
        I: RowIndex + fmt::Display,
    {
        let mut result = Vec::new();
        for row in rows.into_iter() {
            let object: ComponentId = row.try_get(&idx)?;
            result.push(object);
        }
        Ok(result)
    }
}

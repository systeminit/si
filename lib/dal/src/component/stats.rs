//! This module contains [`ComponentStats`].

use serde::Deserialize;
use serde::Serialize;
use telemetry::prelude::*;
use tokio_postgres::Row;

use crate::component::ComponentResult;
use crate::{ComponentId, DalContext};

const LIST_MODIFIED: &str = include_str!("../queries/component_stats_list_modified.sql");
const LIST_ADDED: &str = include_str!("../queries/component_stats_list_added.sql");
const LIST_DELETED: &str = include_str!("../queries/component_stats_list_deleted.sql");

/// A collection of statistics for [`Components`](crate::Component) in the current
/// [`ChangeSet`](crate::ChangeSet).
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ComponentStats {
    added: Vec<ComponentStatsGroup>,
    deleted: Vec<ComponentStatsGroup>,
    modified: Vec<ComponentStatsGroup>,
}

impl ComponentStats {
    pub async fn new(ctx: &DalContext<'_, '_>) -> ComponentResult<Self> {
        let component_stats = if ctx.visibility().is_head() {
            Self::default()
        } else {
            let added = Self::list_added(ctx).await?;
            let deleted = Self::list_deleted(ctx).await?;
            let modified = Self::list_modified(ctx).await?;
            Self {
                added,
                deleted,
                modified,
            }
        };
        Ok(component_stats)
    }

    #[instrument(skip_all)]
    async fn list_added(ctx: &DalContext<'_, '_>) -> ComponentResult<Vec<ComponentStatsGroup>> {
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
        ComponentStatsGroup::new_from_rows(rows)
    }

    #[instrument(skip_all)]
    async fn list_deleted(ctx: &DalContext<'_, '_>) -> ComponentResult<Vec<ComponentStatsGroup>> {
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
        ComponentStatsGroup::new_from_rows(rows)
    }

    #[instrument(skip_all)]
    async fn list_modified(ctx: &DalContext<'_, '_>) -> ComponentResult<Vec<ComponentStatsGroup>> {
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
        ComponentStatsGroup::new_from_rows(rows)
    }
}

/// An individual unit containing metadata for each "counting" statistic.
#[derive(Deserialize, Serialize, Debug)]
pub struct ComponentStatsGroup {
    component_id: ComponentId,
    component_name: String,
}

impl ComponentStatsGroup {
    pub fn new_from_rows(rows: Vec<Row>) -> ComponentResult<Vec<Self>> {
        let mut result = Vec::new();
        for row in rows.into_iter() {
            let component_id: ComponentId = row.try_get("component_id")?;
            let component_name: String = row.try_get("component_name")?;
            result.push(Self {
                component_id,
                component_name,
            });
        }
        Ok(result)
    }
}

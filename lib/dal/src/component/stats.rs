//! This module contains [`ComponentStats`].

use serde::Deserialize;
use serde::Serialize;
use si_data::PgRow;
use strum_macros::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;

use crate::component::ComponentResult;
use crate::{ComponentId, DalContext};

const LIST_MODIFIED: &str = include_str!("../queries/component_stats_list_modified.sql");
const LIST_ADDED: &str = include_str!("../queries/component_stats_list_added.sql");
const LIST_DELETED: &str = include_str!("../queries/component_stats_list_deleted.sql");

/// A collection of statistics for [`Components`](crate::Component) in the current
/// [`ChangeSet`](crate::ChangeSet).
#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComponentStats {
    stats: Vec<ComponentStatsGroup>,
}

impl ComponentStats {
    pub async fn new(ctx: &DalContext) -> ComponentResult<Self> {
        let component_stats = if ctx.visibility().is_head() {
            Self::default()
        } else {
            let added = Self::list_added(ctx).await?;
            let deleted = Self::list_deleted(ctx).await?;
            let modified = Self::list_modified(ctx).await?;

            let mut stats = Vec::new();
            stats.extend(added);
            stats.extend(deleted);
            stats.extend(modified);
            Self { stats }
        };
        Ok(component_stats)
    }

    #[instrument(skip_all)]
    async fn list_added(ctx: &DalContext) -> ComponentResult<Vec<ComponentStatsGroup>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_ADDED,
                &[ctx.read_tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;
        ComponentStatsGroup::new_from_rows(rows, ComponentStatus::Added)
    }

    #[instrument(skip_all)]
    async fn list_deleted(ctx: &DalContext) -> ComponentResult<Vec<ComponentStatsGroup>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_DELETED,
                &[ctx.read_tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;
        ComponentStatsGroup::new_from_rows(rows, ComponentStatus::Deleted)
    }

    #[instrument(skip_all)]
    async fn list_modified(ctx: &DalContext) -> ComponentResult<Vec<ComponentStatsGroup>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_MODIFIED,
                &[ctx.read_tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;
        ComponentStatsGroup::new_from_rows(rows, ComponentStatus::Modified)
    }
}

/// An enum representing the status of the [`Component`](crate::Component) in the
/// [`ChangeSet`](crate::ChangeSet).
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display, EnumString, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ComponentStatus {
    Added,
    Deleted,
    Modified,
}

/// An individual unit containing metadata for each "counting" statistic.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComponentStatsGroup {
    component_id: ComponentId,
    component_name: String,
    component_status: ComponentStatus,
}

impl ComponentStatsGroup {
    pub fn new_from_rows(
        rows: Vec<PgRow>,
        component_status: ComponentStatus,
    ) -> ComponentResult<Vec<Self>> {
        let mut result = Vec::new();
        for row in rows.into_iter() {
            let component_id: ComponentId = row.try_get("component_id")?;
            let component_name: String = row.try_get("component_name")?;

            // TODO(nick): don't move the enum.
            result.push(Self {
                component_id,
                component_name,
                component_status: component_status.clone(),
            });
        }
        Ok(result)
    }
}

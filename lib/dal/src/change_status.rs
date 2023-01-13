//! This module contains [`ComponentChangeStatus`].

use serde::Deserialize;
use serde::Serialize;
use si_data_pg::{PgError, PgRow};
use strum_macros::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::edge::EdgeId;
use crate::{ComponentId, DalContext};

const LIST_MODIFIED_COMPONENTS: &str =
    include_str!("queries/change_status/list_modified_components.sql");
const LIST_ADDED_COMPONENTS: &str = include_str!("queries/change_status/list_added_components.sql");
const LIST_DELETED_COMPONENTS: &str =
    include_str!("queries/change_status/list_deleted_components.sql");
const LIST_EDGE_CHANGE_STATUSES: &str =
    include_str!("queries/change_status/list_edge_change_statuses.sql");

#[derive(Error, Debug)]
pub enum ChangeStatusError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
}

pub type ChangeStatusResult<T> = Result<T, ChangeStatusError>;

/// An enum representing the change_status of an entity in the [`ChangeSet`](crate::ChangeSet).
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Display, EnumString, AsRefStr)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ChangeStatus {
    Added,
    Deleted,
    Modified,
    Unmodified,
}

/// A collection of statistics for [`Components`](crate::Component) in the current
/// [`ChangeSet`](crate::ChangeSet).
#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComponentChangeStatus {
    stats: Vec<ComponentChangeStatusGroup>,
}

impl ComponentChangeStatus {
    pub async fn new(ctx: &DalContext) -> ChangeStatusResult<Self> {
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
    pub async fn list_added(
        ctx: &DalContext,
    ) -> ChangeStatusResult<Vec<ComponentChangeStatusGroup>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_ADDED_COMPONENTS,
                &[ctx.read_tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;
        ComponentChangeStatusGroup::new_from_rows(rows, ChangeStatus::Added)
    }

    #[instrument(skip_all)]
    pub async fn list_deleted(
        ctx: &DalContext,
    ) -> ChangeStatusResult<Vec<ComponentChangeStatusGroup>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_DELETED_COMPONENTS,
                &[ctx.read_tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;
        ComponentChangeStatusGroup::new_from_rows(rows, ChangeStatus::Deleted)
    }

    #[instrument(skip_all)]
    pub async fn list_modified(
        ctx: &DalContext,
    ) -> ChangeStatusResult<Vec<ComponentChangeStatusGroup>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_MODIFIED_COMPONENTS,
                &[ctx.read_tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;
        ComponentChangeStatusGroup::new_from_rows(rows, ChangeStatus::Modified)
    }
}

/// An individual unit containing metadata for each "counting" statistic.
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComponentChangeStatusGroup {
    pub component_id: ComponentId,
    component_name: String,
    pub component_status: ChangeStatus,
}

impl ComponentChangeStatusGroup {
    pub fn new_from_rows(
        rows: Vec<PgRow>,
        component_status: ChangeStatus,
    ) -> ChangeStatusResult<Vec<Self>> {
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

pub struct EdgeChangeStatus;

pub struct EdgeChangeStatusItem {
    pub edge_id: EdgeId,
    pub status: ChangeStatus,
}

impl EdgeChangeStatus {
    pub async fn list(ctx: &DalContext) -> ChangeStatusResult<Vec<EdgeChangeStatusItem>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_EDGE_CHANGE_STATUSES,
                &[ctx.read_tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;

        let mut result = Vec::new();
        for row in rows.into_iter() {
            let edge_id: EdgeId = row.try_get("id")?;
            let status: String = row.try_get("status")?;

            // TODO(nick): don't move the enum.
            result.push(EdgeChangeStatusItem {
                edge_id,
                status: match status.as_str() {
                    "added" => ChangeStatus::Added,
                    "deleted" => ChangeStatus::Deleted,
                    _ => ChangeStatus::Unmodified,
                },
            });
        }

        Ok(result)
    }
}

//! This module contains [`ComponentChangeStatus`].

use serde::Deserialize;
use serde::Serialize;
use si_data_pg::{PgError, PgRow};
use strum::{AsRefStr, Display, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::standard_model::objects_from_rows;
use crate::TransactionsError;
use crate::{ComponentId, DalContext, Edge, StandardModelError};

const LIST_MODIFIED_COMPONENTS: &str =
    include_str!("queries/change_status/list_modified_components.sql");
const LIST_ADDED_COMPONENTS: &str = include_str!("queries/change_status/list_added_components.sql");
const LIST_DELETED_COMPONENTS: &str =
    include_str!("queries/change_status/list_deleted_components.sql");
const LIST_DELETED_EDGES: &str = include_str!("queries/change_status/edges_list_deleted.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ChangeStatusError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Tranactions(#[from] TransactionsError),
}

pub type ChangeStatusResult<T> = Result<T, ChangeStatusError>;

/// An enum representing the change_status of an entity in the [`ChangeSet`](crate::ChangeSet).
#[remain::sorted]
#[derive(
    Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy, Display, EnumString, AsRefStr,
)]
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
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
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

    #[instrument(level = "debug", skip_all)]
    pub async fn list_added(
        ctx: &DalContext,
    ) -> ChangeStatusResult<Vec<ComponentChangeStatusGroup>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_ADDED_COMPONENTS,
                &[ctx.tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;
        ComponentChangeStatusGroup::new_from_rows(rows, ChangeStatus::Added)
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn list_deleted(
        ctx: &DalContext,
    ) -> ChangeStatusResult<Vec<ComponentChangeStatusGroup>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_DELETED_COMPONENTS,
                &[ctx.tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;
        ComponentChangeStatusGroup::new_from_rows(rows, ChangeStatus::Deleted)
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn list_modified(
        ctx: &DalContext,
    ) -> ChangeStatusResult<Vec<ComponentChangeStatusGroup>> {
        if ctx.visibility().is_head() {
            return Ok(vec![]);
        }

        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_MODIFIED_COMPONENTS,
                &[ctx.tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;
        ComponentChangeStatusGroup::new_from_rows(rows, ChangeStatus::Modified)
    }
}

/// An individual unit containing metadata for each "counting" statistic.
#[derive(Deserialize, Serialize, Debug, Clone)]
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
            let component_name: Option<String> = row.try_get("component_name")?;
            let component_name = component_name.unwrap_or_else(|| "".to_owned());

            result.push(Self {
                component_id,
                component_name,
                component_status,
            });
        }
        Ok(result)
    }
}

pub struct EdgeChangeStatus;

impl EdgeChangeStatus {
    pub async fn list_deleted(ctx: &DalContext) -> ChangeStatusResult<Vec<Edge>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_DELETED_EDGES,
                &[ctx.tenancy(), &ctx.visibility().change_set_pk],
            )
            .await?;

        Ok(objects_from_rows(rows)?)
    }
}

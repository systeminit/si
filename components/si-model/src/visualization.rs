use std::{convert::TryFrom, convert::TryInto, num::TryFromIntError};

use chrono::{prelude::*, Duration};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::PgTxn;

use crate::{ChangeSet, EntityError, Resource, WorkflowRun};

const CHANGE_SET_APPLIED_BY_APPLICATION: &str =
    include_str!("./queries/change_set_applied_by_application.sql");
const CHANGE_SET_OPEN_BY_APPLICATION: &str =
    include_str!("queries/change_set_open_by_application.sql");
const CHANGE_SET_NEW_NODES: &str = include_str!("queries/change_set_new_nodes.sql");
const CHANGE_SET_MODIFIED_NODES: &str = include_str!("queries/change_set_modified_nodes.sql");
const CHANGE_SET_DELETED_NODES: &str = include_str!("queries/change_set_deleted_nodes.sql");
const WORKFLOW_RUNS_SERVICE_DEPLOY_BY_APPLICATION: &str =
    include_str!("./queries/workflow_runs_service_deploy_by_application.sql");
const RESOURCE_FOR_VISUALIZATION: &str = include_str!("queries/resource_for_visualization.sql");

#[derive(Error, Debug)]
pub enum VisualizationError {
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("entity error: {0}")]
    Entity(#[from] EntityError),
    #[error("integer conversion error: {0}")]
    TryFromIntError(#[from] TryFromIntError),
}

pub type VisualizationResult<T> = Result<T, VisualizationError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySummary {
    labels: Vec<String>,
    apply_data: Vec<u32>,
    deploy_data: Vec<u32>,
}

pub async fn activity_summary(
    txn: &PgTxn<'_>,
    application_id: impl AsRef<str>,
) -> VisualizationResult<ActivitySummary> {
    let current_date = Utc::now();
    let historical_days: usize = 7;
    let seven_days = Duration::days(historical_days as i64);
    let seven_days_ago = current_date - seven_days;

    let mut labels: Vec<String> = vec![];
    for day in 0..historical_days {
        let this_day = current_date - Duration::days(day as i64);
        let weekday = this_day.weekday();
        let weekday_str = match weekday {
            Weekday::Mon => "Mo",
            Weekday::Tue => "Tu",
            Weekday::Wed => "We",
            Weekday::Thu => "Th",
            Weekday::Fri => "Fr",
            Weekday::Sat => "Sa",
            Weekday::Sun => "Su",
        };
        labels.push(weekday_str.to_string());
    }
    labels.reverse();

    let mut apply_data: Vec<u32> = vec![0, 0, 0, 0, 0, 0, 0];
    let mut deploy_data: Vec<u32> = vec![0, 0, 0, 0, 0, 0, 0];

    let application_id = application_id.as_ref();
    let rows = txn
        .query(CHANGE_SET_APPLIED_BY_APPLICATION, &[&application_id])
        .await?;
    for row in rows.into_iter() {
        let json: serde_json::Value = row.try_get("object")?;
        let change_set: ChangeSet = serde_json::from_value(json)?;
        if let Some(updated_at) = change_set.si_storable.updated_at {
            if updated_at > seven_days_ago {
                let how_far_back = current_date - updated_at;
                let days_distant = how_far_back.num_days();
                if days_distant < 0 || days_distant > 7 {
                    dbg!("um, we fucked up the date calcuations");
                } else {
                    let days_distant: usize = usize::try_from(days_distant)?;
                    let current_count = apply_data[days_distant];
                    apply_data[days_distant] = current_count + 1;
                }
            }
        }
    }
    apply_data.reverse();

    let rows = txn
        .query(
            WORKFLOW_RUNS_SERVICE_DEPLOY_BY_APPLICATION,
            &[&application_id],
        )
        .await?;
    for row in rows.into_iter() {
        let json: serde_json::Value = row.try_get("object")?;
        let workflow_run: WorkflowRun = serde_json::from_value(json)?;
        if let Some(created_at) = workflow_run.si_storable.created_at {
            if created_at > seven_days_ago {
                let how_far_back = current_date - created_at;
                let days_distant = how_far_back.num_days();
                if days_distant < 0 || days_distant > 7 {
                    dbg!("um, we fucked up the date calcuations");
                } else {
                    let days_distant: usize = usize::try_from(days_distant)?;
                    let current_count = deploy_data[days_distant];
                    deploy_data[days_distant] = current_count + 1;
                }
            }
        }
    }
    deploy_data.reverse();

    Ok(ActivitySummary {
        apply_data,
        deploy_data,
        labels,
    })
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChangesSummary {
    pub open_change_set_count: u32,
    pub current_change_set: Option<CurrentChangeSetSummary>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurrentChangeSetSummary {
    pub new_nodes: u32,
    pub deleted_nodes: u32,
    pub modified_nodes: u32,
}

pub async fn changes_summary(
    txn: &PgTxn<'_>,
    application_id: impl AsRef<str>,
    change_set_id: Option<impl AsRef<str>>,
) -> VisualizationResult<ChangesSummary> {
    let application_id = application_id.as_ref();
    let rows = txn
        .query(CHANGE_SET_OPEN_BY_APPLICATION, &[&application_id])
        .await?;
    let open_change_set_count: u32 = rows.len().try_into()?;
    let mut current_change_set: Option<CurrentChangeSetSummary> = None;

    if let Some(change_set_id) = change_set_id {
        let change_set_id = change_set_id.as_ref();
        let rows = txn.query(CHANGE_SET_NEW_NODES, &[&change_set_id]).await?;
        let new_nodes: u32 = rows.len().try_into()?;
        let rows = txn
            .query(CHANGE_SET_MODIFIED_NODES, &[&change_set_id])
            .await?;
        let modified_nodes: u32 = rows.len().try_into()?;
        let rows = txn
            .query(CHANGE_SET_DELETED_NODES, &[&change_set_id])
            .await?;
        let deleted_nodes: u32 = rows.len().try_into()?;
        current_change_set = Some(CurrentChangeSetSummary {
            new_nodes,
            modified_nodes,
            deleted_nodes,
        });
    }
    Ok(ChangesSummary {
        open_change_set_count,
        current_change_set,
    })
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSummary {
    resources: Vec<Resource>,
}

pub async fn resource_summary(
    txn: &PgTxn<'_>,
    application_id: impl AsRef<str>,
    system_id: impl AsRef<str>,
    entity_types: Vec<&str>,
) -> VisualizationResult<ResourceSummary> {
    let application_id = application_id.as_ref();
    let system_id = system_id.as_ref();
    let mut resources = vec![];
    let rows = txn
        .query(
            RESOURCE_FOR_VISUALIZATION,
            &[&application_id, &system_id, &entity_types],
        )
        .await?;
    for row in rows.into_iter() {
        let json: serde_json::Value = row.try_get("object")?;
        let resource: Resource = serde_json::from_value(json)?;
        resources.push(resource);
    }
    Ok(ResourceSummary { resources })
}

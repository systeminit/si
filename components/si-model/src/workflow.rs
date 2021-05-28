use chrono::Utc;
use futures::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror::Error;
use tokio::sync::oneshot;

use si_data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};

use crate::{
    workflow::selector::SelectionEntry, workflow::step::WorkflowRunStepState, EdgeError, Entity,
    LodashError, MinimalStorable, ResourceError, SiStorable, Veritech, VeritechError, Workspace,
};

const WORKFLOW_GET_BY_NAME: &str = include_str!("./queries/workflow_get_by_name.sql");
const WORKFLOW_ACTION_LIST_ALL: &str = include_str!("./queries/workflow_action_list_all.sql");
const WORKFLOW_ACTION_LIST_ALL_FOR_SCHEMATIC: &str =
    include_str!("./queries/workflow_action_list_all_for_schematic.sql");
const WORKFLOW_RUN_STEPS_ALL: &str = include_str!("./queries/workflow_run_steps_all.sql");
const WORKFLOW_RUN_STEP_ENTITIES_ALL: &str =
    include_str!("./queries/workflow_run_step_entities_all.sql");

pub mod selector;
pub mod step;
pub mod variable;

use crate::workflow::step::Step;

use self::{
    selector::Selector,
    step::{WorkflowRunStep, WorkflowRunStepEntity},
};

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("pg error: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("veritech error: {0}")]
    Veritech(#[from] VeritechError),
    #[error("lodash error: {0}")]
    Lodash(#[from] LodashError),
    #[error("no selector or implicit entity provided; invalid step!")]
    NoSelectorOrEntity,
    #[error("no {0} value found in {1} for path {2}")]
    NoValue(String, String, String),
    #[error("expected {0} recevied {:?}")]
    WrongType(String, serde_json::Value),
    #[error("invalid strategy: {0}")]
    InvalidStrategy(String),
    #[error("no inputs when they were required")]
    NoInputs,
    #[error("no system is provided, but is required!")]
    SystemRequired,
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("entity error: {0}")]
    Entity(String),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
    #[error("Selector requires root entity selection (for now)")]
    SelectorWithoutRootEntity,
    #[error("Selector requested properties, but has none for the system")]
    NoPropertiesForSystem,
    #[error("Selector requested an entity from property {0:?}, but it was not found")]
    PropertyNotFound(Vec<String>),
    #[error("Selector found a value in property {0:?}, but it was not a string!")]
    PropertyNotAString(Vec<String>),
    #[error("Edge Kind is required in a selector, but it was not provided")]
    EdgeKindMissing,
    #[error("Depth is required in a selector, but it was not provided")]
    DepthMissing,
    #[error("Direction is required in a selector, but it was not provided")]
    DirectionMissing,
    #[error("No name when one was required in inputs")]
    NoNameInInputs,
    #[error("Tokio oneshot recv error: {0}")]
    TokioOneshotRecv(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("Workflow named '{0}' was not found")]
    WorkflowNotFound(String),
    #[error("The step failed")]
    WorkflowStepFailed,
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Display, Clone)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WorkflowRunState {
    Invoked,
    Running,
    Success,
    Failure,
    Unknown,
}

impl Default for WorkflowRunState {
    fn default() -> Self {
        Self::Invoked
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunListItem {
    workflow_run: WorkflowRun,
    steps: Vec<WorkflowRunListStepsItem>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunListStepsItem {
    step: WorkflowRunStep,
    step_entities: Vec<WorkflowRunStepEntity>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRun {
    pub id: String,
    pub start_unix_timestamp: i64,
    pub start_timestamp: String,
    pub end_unix_timestamp: Option<i64>,
    pub end_timestamp: Option<String>,
    pub state: WorkflowRunState,
    pub workflow_id: String,
    pub workflow_name: String,
    pub data: WorkflowData,
    pub ctx: WorkflowContext,
    pub si_storable: SiStorable,
}

impl WorkflowRun {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        workflow: &Workflow,
        ctx: WorkflowContext,
    ) -> WorkflowResult<Self> {
        let workflow_data = serde_json::to_value(workflow.data.clone())?;
        let ctx = serde_json::to_value(ctx)?;
        let state = WorkflowRunState::default();
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);

        let row = txn
            .query_one(
                "SELECT object FROM workflow_run_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &workflow.id,
                    &workflow.name,
                    &workflow_data,
                    &ctx,
                    &state.to_string(),
                    &timestamp,
                    &unix_timestamp,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Self = serde_json::from_value(json)?;

        Ok(object)
    }

    pub async fn list_actions_for_schematic(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        action_name: Option<impl AsRef<str>>,
    ) -> WorkflowResult<Vec<WorkflowRunListItem>> {
        let entity_id = entity_id.as_ref();
        let system_id = system_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let rows = if let Some(action_name) = action_name {
            let _action_name = action_name.as_ref();
            // TODO: Don't leave this like that ;)
            let rows = txn
                .query(
                    WORKFLOW_ACTION_LIST_ALL_FOR_SCHEMATIC,
                    &[&entity_id, &system_id, &workspace_id],
                )
                .await?;
            rows
        } else {
            let rows = txn
                .query(
                    WORKFLOW_ACTION_LIST_ALL_FOR_SCHEMATIC,
                    &[&entity_id, &system_id, &workspace_id],
                )
                .await?;
            rows
        };
        let mut results: Vec<WorkflowRunListItem> = vec![];

        for row in rows.into_iter() {
            let workflow_run_json: serde_json::Value = row.try_get("object")?;
            let workflow_run: WorkflowRun = serde_json::from_value(workflow_run_json)?;
            results.push(WorkflowRunListItem {
                workflow_run,
                steps: vec![],
            });
        }
        Ok(results)
    }

    pub async fn list_actions(
        txn: &PgTxn<'_>,
        entity_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
        action_name: Option<impl AsRef<str>>,
    ) -> WorkflowResult<Vec<WorkflowRunListItem>> {
        let entity_id = entity_id.as_ref();
        let system_id = system_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let rows = if let Some(action_name) = action_name {
            let _action_name = action_name.as_ref();
            // TODO: Don't leave this like that ;)
            let rows = txn
                .query(
                    WORKFLOW_ACTION_LIST_ALL,
                    &[&entity_id, &system_id, &workspace_id],
                )
                .await?;
            rows
        } else {
            let rows = txn
                .query(
                    WORKFLOW_ACTION_LIST_ALL,
                    &[&entity_id, &system_id, &workspace_id],
                )
                .await?;
            rows
        };
        let mut results: Vec<WorkflowRunListItem> = vec![];

        for row in rows.into_iter() {
            let workflow_run_json: serde_json::Value = row.try_get("object")?;
            let workflow_run: WorkflowRun = serde_json::from_value(workflow_run_json)?;
            let workflow_run_steps_rows = txn
                .query(WORKFLOW_RUN_STEPS_ALL, &[&workflow_run.id])
                .await?;
            let mut wrs_results: Vec<WorkflowRunListStepsItem> = vec![];
            for wrs_row in workflow_run_steps_rows.into_iter() {
                let workflow_run_step_json: serde_json::Value = wrs_row.try_get("object")?;
                let workflow_run_step: WorkflowRunStep =
                    serde_json::from_value(workflow_run_step_json)?;
                let workflow_run_step_entity_rows = txn
                    .query(WORKFLOW_RUN_STEP_ENTITIES_ALL, &[&workflow_run_step.id])
                    .await?;
                let mut wrse_results: Vec<WorkflowRunStepEntity> = vec![];
                for wrse_row in workflow_run_step_entity_rows.into_iter() {
                    let workflow_run_step_entity_json: serde_json::Value =
                        wrse_row.try_get("object")?;
                    let workflow_run_step_entity: WorkflowRunStepEntity =
                        serde_json::from_value(workflow_run_step_entity_json)?;
                    wrse_results.push(workflow_run_step_entity);
                }
                wrs_results.push(WorkflowRunListStepsItem {
                    step: workflow_run_step,
                    step_entities: wrse_results,
                });
            }
            results.push(WorkflowRunListItem {
                workflow_run,
                steps: wrs_results,
            });
        }
        Ok(results)
    }

    pub async fn invoke(
        &self,
        pg: PgPool,
        nats_conn: NatsConn,
        veritech: Veritech,
        wait_channel: Option<oneshot::Sender<WorkflowResult<()>>>,
    ) -> WorkflowResult<()> {
        let workflow_name = self.workflow_name.clone();
        let workflow_run = self.clone();
        tokio::spawn(async move {
            let result = workflow_run.invoke_task(pg, nats_conn, veritech).await;

            if let Err(ref err) = result {
                dbg!("invoking workflow {} failed: {:?}", &workflow_name, &err);
                dbg!(&workflow_name);
                dbg!(&err);
            }
            if let Some(wait_channel) = wait_channel {
                let _ = wait_channel.send(result);
            }
        });
        Ok(())
    }

    async fn invoke_task(
        mut self,
        pg: PgPool,
        nats_conn: NatsConn,
        veritech: Veritech,
    ) -> WorkflowResult<()> {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        self.state = WorkflowRunState::Running;
        self.save(&txn, &nats).await?;

        txn.commit().await?;
        nats.commit().await?;

        let mut final_state = WorkflowRunState::Success;
        let mut final_error: Option<WorkflowError> = None;
        for step in self.data.steps() {
            match step.run(&pg, &nats_conn, &veritech, &mut self).await {
                Ok(workflow_run_step) => {
                    if workflow_run_step.state == WorkflowRunStepState::Failure {
                        final_error = Some(WorkflowError::WorkflowStepFailed);
                        final_state = WorkflowRunState::Failure;
                        break;
                    }
                }
                Err(e) => {
                    final_error = Some(e);
                    final_state = WorkflowRunState::Failure;
                    break;
                }
            }
        }

        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);
        self.end_timestamp = Some(timestamp);
        self.end_unix_timestamp = Some(unix_timestamp);
        self.state = final_state;
        if final_error.is_some() {
            dbg!(&final_error);
        }

        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        self.save(&txn, &nats).await?;

        nats.commit().await?;
        txn.commit().await?;

        if let Some(error) = final_error {
            Err(error)
        } else {
            Ok(())
        }
    }

    pub async fn save(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> WorkflowResult<()> {
        let json = serde_json::to_value(&self)?;

        let row = txn
            .query_one("SELECT object FROM workflow_run_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;

        let mut updated: Self = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }

    pub async fn refresh(&mut self, txn: &PgTxn<'_>) -> WorkflowResult<()> {
        let json = serde_json::to_value(&self)?;

        let row = txn
            .query_one(
                "SELECT obj AS object FROM workflow_runs WHERE si_id = $1",
                &[&self.id],
            )
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        let mut updated: Self = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowForAction {
    name: String,
    title: String,
    description: String,
    steps: Vec<Step>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTop {
    name: String,
    title: String,
    description: String,
    steps: Vec<Step>,
    args: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum WorkflowData {
    Action(WorkflowForAction),
    //Top(WorkflowTop),
}

impl WorkflowData {
    // TODO: This is so unneccsary, but going to work fine.
    pub fn steps(&self) -> Vec<Step> {
        match self {
            WorkflowData::Action(w) => w.steps.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    id: String,
    name: String,
    data: WorkflowData,
    si_storable: MinimalStorable,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadWorkflowRequest {
    doit: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadWorkflowReply {
    workflows: Vec<WorkflowData>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowContext {
    pub dry_run: bool,
    pub entity: Option<Entity>,
    pub system: Option<Entity>,
    pub selection: Vec<SelectionEntry>,
    pub strategy: Option<String>,
    pub fail_if_missing: Option<bool>,
    pub inputs: Option<serde_json::Value>,
    pub args: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub store: Option<serde_json::Value>,
    pub workspace: Workspace,
}

impl WorkflowContext {
    pub async fn for_step(&mut self, pg: &PgPool, step: &Step) -> WorkflowResult<()> {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;

        // First, evaluate any selector. If no selector, use the entity from
        // the workflow run context. If there isn't one, fail with a message
        // requiring a selector.
        let selector_entries = match step.selector() {
            Some(selector) => selector.resolve(&txn, &self).await?,
            None => Selector::new().resolve(&txn, &self).await?,
        };
        self.selection = selector_entries;

        // Then, evaluate the strategy for the step. If it isn't one of the
        // valid values after computing, fail.
        let strategy = step.strategy(&self)?;
        match strategy.as_ref() {
            "linear" => {}
            _ => return Err(WorkflowError::InvalidStrategy(strategy)),
        }
        self.strategy = Some(strategy);

        // Evaluate wether we should fail if the step is missing on the
        // entities matched by the selector
        let fail_if_missing = step.fail_if_missing(&self)?;
        self.fail_if_missing = Some(fail_if_missing);

        // Then, evaluate all the inputs and resolve them. Their values should
        // be added to the context as 'inputs'.
        let inputs = step.inputs(&self)?;
        self.inputs = Some(inputs);

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum InvokeWorkflowProtocol {
    Start(String),
    Finished,
}

impl Workflow {
    pub async fn load_builtins(pg: &PgPool, veritech: &Veritech) -> WorkflowResult<()> {
        let reply: LoadWorkflowReply = veritech
            .send_sync("loadWorkflows", LoadWorkflowRequest { doit: true })
            .await?;

        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        for workflow_data in reply.workflows {
            Self::upsert_from(&txn, &workflow_data).await?;
        }
        txn.commit().await?;

        Ok(())
    }

    pub async fn upsert_from(
        txn: &PgTxn<'_>,
        workflow_data: &WorkflowData,
    ) -> WorkflowResult<Self> {
        let json = serde_json::to_value(workflow_data)?;
        let row = txn
            .query_one(
                "SELECT object from workflow_create_or_update_v1($1)",
                &[&json],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;

        Ok(object)
    }

    pub fn entity_and_action_name_to_workflow_name(
        entity: &Entity,
        action_name: impl AsRef<str>,
    ) -> &'static str {
        let action_name = action_name.as_ref();
        let s = match (&entity.entity_type[..], action_name) {
            ("application", "deploy") => "application:deploy",
            ("service", "deploy") => "service:deploy",
            ("service", "terminate") => "service:terminate",
            ("kubernetesCluster", "deploy") => "kubernetesCluster:deploy",
            ("kubernetesService", "deploy") => "kubernetesService:deploy",
            ("kubernetesService", "terminate") => "kubernetesService:terminate",
            (_, "apply") => "kubernetesApply",
            (_, "delete") => "kubernetesDelete",
            (_, _) => "universal:deploy",
        };
        s
    }

    pub async fn get_by_name(txn: &PgTxn<'_>, name: impl AsRef<str>) -> WorkflowResult<Self> {
        let name = name.as_ref();

        let row = txn
            .query_opt(WORKFLOW_GET_BY_NAME, &[&name])
            .await?
            .ok_or_else(|| WorkflowError::WorkflowNotFound(name.to_string()))?;
        let object: serde_json::Value = row.try_get("object")?;
        let workflow: Self = serde_json::from_value(object)?;

        Ok(workflow)
    }

    pub fn invoke(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        ctx: WorkflowContext,
    ) -> BoxFuture<WorkflowResult<WorkflowRun>> {
        self.inner_invoke(pg, nats_conn, veritech, ctx, None)
    }

    pub async fn invoke_and_wait(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        ctx: WorkflowContext,
    ) -> WorkflowResult<WorkflowRun> {
        let (tx, rx) = oneshot::channel();
        let result = self
            .inner_invoke(pg, nats_conn, veritech, ctx, Some(tx))
            .await?;
        let _ = rx.await?;
        Ok(result)
    }

    fn inner_invoke(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        ctx: WorkflowContext,
        wait_channel: Option<oneshot::Sender<WorkflowResult<()>>>,
    ) -> BoxFuture<WorkflowResult<WorkflowRun>> {
        let pg = pg.clone();
        let nats_conn = nats_conn.clone();
        let veritech = veritech.clone();
        async move {
            let mut conn = pg.pool.get().await?;
            let txn = conn.transaction().await?;
            let nats = nats_conn.transaction();

            let workflow_run = WorkflowRun::new(&txn, &nats, &self, ctx).await?;

            txn.commit().await?;
            nats.commit().await?;

            workflow_run
                .invoke(
                    pg.clone(),
                    nats_conn.clone(),
                    veritech.clone(),
                    wait_channel,
                )
                .await?;
            Ok(workflow_run)
        }
        .boxed()
    }
}

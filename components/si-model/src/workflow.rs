use chrono::Utc;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use thiserror::Error;
use tokio::sync::oneshot;

use si_data::{NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};

use crate::{
    Edge, EdgeError, EdgeKind, Entity, LodashError, MinimalStorable, Resource, ResourceError,
    SiStorable, Veritech, VeritechError, Workspace,
};

const WORKFLOW_GET_BY_NAME: &str = include_str!("./queries/workflow_get_by_name.sql");
const WORKFLOW_ACTION_LIST_ALL: &str = include_str!("./queries/workflow_action_list_all.sql");
const WORKFLOW_RUN_STEPS_ALL: &str = include_str!("./queries/workflow_run_steps_all.sql");
const WORKFLOW_RUN_STEP_ENTITIES_ALL: &str =
    include_str!("./queries/workflow_run_step_entities_all.sql");

pub mod step;
pub mod variable;

use crate::workflow::step::Step;

use self::step::{WorkflowRunStep, WorkflowRunStepEntity};

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
            let action_name = action_name.as_ref();
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
                dbg!("invoking workflow {} failed: {:?}", workflow_name, err);
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

        let mut final_state = WorkflowRunState::Success;
        let mut final_error: Option<WorkflowError> = None;
        for step in self.data.steps() {
            match step.run(&pg, &nats_conn, &veritech, &mut self).await {
                Ok(_) => {}
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
        let txn = conn.transaction().await?;
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
pub struct SelectionEntryPredecessor {
    entity: Entity,
    resource: Resource,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectionEntry {
    entity: Entity,
    resource: Resource,
    predecessors: Vec<SelectionEntryPredecessor>,
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
    pub async fn for_step(
        &mut self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        step: &Step,
    ) -> WorkflowResult<()> {
        // First, evaluate any selector. If no selector, use the entity from
        // the workflow run context. If there isn't one, fail with a message
        // requiring a selector.
        let selector_entities = match step.selector() {
            Some(_selector) => {
                todo!("implement selector resolution");
            }
            None => match &self.entity {
                Some(entity) => {
                    let mut conn = pg.pool.get().await?;
                    let txn = conn.transaction().await?;
                    let nats = nats_conn.transaction();
                    let system_id = self
                        .system
                        .as_ref()
                        .ok_or(WorkflowError::SystemRequired)?
                        .id
                        .clone();
                    let predecessor_edges = Edge::direct_predecessor_edges_by_object_id(
                        &txn,
                        &EdgeKind::Configures,
                        &entity.id,
                    )
                    .await?;
                    let mut predecessors: Vec<SelectionEntryPredecessor> = Vec::new();
                    for edge in predecessor_edges {
                        let edge_entity = Entity::for_head(&txn, &edge.tail_vertex.object_id)
                            .await
                            .map_err(|e| WorkflowError::Entity(e.to_string()))?;
                        let predecessor_resource = Resource::for_system(
                            &txn,
                            &nats,
                            &edge_entity.id,
                            &system_id,
                            &self.workspace.id,
                        )
                        .await?;
                        let predecessor = SelectionEntryPredecessor {
                            entity: edge_entity,
                            resource: predecessor_resource,
                        };
                        predecessors.push(predecessor);
                    }
                    let resource = Resource::for_system(
                        &txn,
                        &nats,
                        &entity.id,
                        &system_id,
                        &self.workspace.id,
                    )
                    .await?;
                    vec![SelectionEntry {
                        entity: entity.clone(),
                        resource,
                        predecessors,
                    }]
                }
                None => return Err(WorkflowError::NoSelectorOrEntity),
            },
        };
        self.selection = selector_entities;

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

    pub async fn get_by_name(txn: &PgTxn<'_>, name: impl AsRef<str>) -> WorkflowResult<Self> {
        let name = name.as_ref();

        let row = txn.query_one(WORKFLOW_GET_BY_NAME, &[&name]).await?;
        let object: serde_json::Value = row.try_get("object")?;
        let workflow: Self = serde_json::from_value(object)?;

        Ok(workflow)
    }

    pub async fn invoke(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        ctx: WorkflowContext,
    ) -> WorkflowResult<WorkflowRun> {
        self.inner_invoke(pg, nats_conn, veritech, ctx, None).await
    }

    pub async fn invoke_and_wait(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        ctx: WorkflowContext,
    ) -> WorkflowResult<(WorkflowRun, oneshot::Receiver<WorkflowResult<()>>)> {
        let (tx, rx) = oneshot::channel();
        let result = self
            .inner_invoke(pg, nats_conn, veritech, ctx, Some(tx))
            .await?;
        Ok((result, rx))
    }

    async fn inner_invoke(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        ctx: WorkflowContext,
        wait_channel: Option<oneshot::Sender<WorkflowResult<()>>>,
    ) -> WorkflowResult<WorkflowRun> {
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
}

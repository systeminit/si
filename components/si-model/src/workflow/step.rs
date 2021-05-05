use serde::{Deserialize, Serialize};
use si_data::{NatsConn, NatsTxn, PgPool, PgTxn};
use strum_macros::Display;

use crate::workflow::variable::{VariableArray, VariableBool, VariableScalar};
use crate::workflow::{
    SelectionEntry, WorkflowContext, WorkflowError, WorkflowResult, WorkflowRun,
};
use crate::{workflow::selector::Selector, Workflow};
use crate::{Entity, SiStorable, Veritech, Workspace};
use chrono::Utc;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Display, Clone)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WorkflowRunStepState {
    Running,
    Success,
    Failure,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowRunStep {
    pub id: String,
    pub workflow_run_id: String,
    pub start_unix_timestamp: i64,
    pub start_timestamp: String,
    pub end_unix_timestamp: Option<i64>,
    pub end_timestamp: Option<String>,
    pub state: WorkflowRunStepState,
    pub step: Step,
    pub si_storable: SiStorable,
}

impl WorkflowRunStep {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        workflow_run: &WorkflowRun,
        step: &Step,
    ) -> WorkflowResult<Self> {
        let workflow_run_id = &workflow_run.id[..];
        let workspace_id = &workflow_run.si_storable.workspace_id[..];
        let step = serde_json::to_value(step)?;
        let state = WorkflowRunStepState::Running;
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);

        let row = txn
            .query_one(
                "SELECT object FROM workflow_run_step_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    &workflow_run_id,
                    &step,
                    &state.to_string(),
                    &timestamp,
                    &unix_timestamp,
                    &workspace_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Self = serde_json::from_value(json)?;

        Ok(object)
    }

    pub async fn save(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> WorkflowResult<()> {
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);
        let json = serde_json::to_value(&self)?;

        self.end_timestamp = Some(timestamp);
        self.end_unix_timestamp = Some(unix_timestamp);

        let row = txn
            .query_one("SELECT object FROM workflow_run_step_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;

        let mut updated: Self = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Display, Clone)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WorkflowRunStepEntityState {
    Starting,
    Running,
    Success,
    Failure,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub struct WorkflowRunStepEntity {
    pub id: String,
    pub workflow_run_id: String,
    pub workflow_run_step_id: String,
    pub entity_id: String,
    pub start_unix_timestamp: i64,
    pub start_timestamp: String,
    pub end_unix_timestamp: Option<i64>,
    pub end_timestamp: Option<String>,
    pub state: WorkflowRunStepEntityState,
    pub output: Option<String>,
    pub error: Option<String>,
    pub si_storable: SiStorable,
}

impl WorkflowRunStepEntity {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        entity_id: impl AsRef<str>,
        workflow_run_step: &WorkflowRunStep,
    ) -> WorkflowResult<Self> {
        let workflow_run_id = &workflow_run_step.workflow_run_id[..];
        let workflow_run_step_id = &workflow_run_step.id[..];
        let entity_id = entity_id.as_ref();
        let workspace_id = &workflow_run_step.si_storable.workspace_id[..];
        let state = WorkflowRunStepEntityState::Starting;
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);

        let row = txn
            .query_one(
                "SELECT object FROM workflow_run_step_entity_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &workflow_run_id,
                    &workflow_run_step_id,
                    &entity_id,
                    &state.to_string(),
                    &timestamp,
                    &unix_timestamp,
                    &workspace_id,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Self = serde_json::from_value(json)?;

        Ok(object)
    }

    pub async fn save(&mut self, txn: &PgTxn<'_>, nats: &NatsTxn) -> WorkflowResult<()> {
        let json = serde_json::to_value(&self)?;

        let row = txn
            .query_one(
                "SELECT object FROM workflow_run_step_entity_save_v1($1)",
                &[&json],
            )
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;

        let mut updated: Self = serde_json::from_value(updated_result)?;
        std::mem::swap(self, &mut updated);
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Step {
    Command(StepCommand),
    Action(StepAction),
    //Workflow(StepWorkflow),
}

impl Step {
    pub async fn run(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        workflow_run: &mut WorkflowRun,
    ) -> WorkflowResult<()> {
        workflow_run.ctx.for_step(&pg, &nats_conn, self).await?;
        match self {
            Step::Command(s) => s.run(pg, nats_conn, veritech, workflow_run).await,
            Step::Action(s) => s.run(pg, nats_conn, veritech, workflow_run).await,
        }
    }

    pub fn selector(&self) -> Option<&Selector> {
        let result = match self {
            Step::Command(s) => s.selector.as_ref(),
            Step::Action(s) => s.selector.as_ref(),
        };
        result
    }

    pub fn strategy(&self, ctx: &WorkflowContext) -> WorkflowResult<String> {
        let result = match self {
            Step::Command(s) => s.strategy.as_ref().map_or(String::from("linear"), |f| {
                f.evaluate_as_string(ctx)
                    .map_or(String::from("linear"), |s| s)
            }), //skeeeetchy
            Step::Action(s) => s.strategy.as_ref().map_or(String::from("linear"), |f| {
                f.evaluate_as_string(ctx)
                    .map_or(String::from("linear"), |s| s)
            }),
        };
        Ok(result)
    }

    pub fn inputs(&self, ctx: &WorkflowContext) -> WorkflowResult<serde_json::Value> {
        let result = match self {
            Step::Command(s) => {
                serde_json::json![{ "name": s.inputs.name.evaluate_as_string(ctx)? }]
            }
            Step::Action(s) => {
                serde_json::json![{ "name": s.inputs.name.evaluate_as_string(ctx)? }]
            }
        };
        Ok(result)
    }

    pub fn fail_if_missing(&self, ctx: &WorkflowContext) -> WorkflowResult<bool> {
        let result = match self {
            Step::Command(s) => s
                .fail_if_missing
                .as_ref()
                .map_or(false, |f| f.evaluate_as_bool(ctx).map_or(false, |b| b)), //skeeeetchy
            Step::Action(s) => s
                .fail_if_missing
                .as_ref()
                .map_or(false, |f| f.evaluate_as_bool(ctx).map_or(false, |b| b)), //skeeeetchy
        };
        Ok(result)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepCommandInputs {
    pub name: VariableScalar,
    pub args: Option<VariableArray>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommandRequest<'a> {
    inputs: &'a serde_json::Value,
    selection: &'a SelectionEntry,
    system: Option<&'a Entity>,
    workspace: &'a Workspace,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum CommandProtocol {
    Start(bool),
    Output(CommandOutput),
    Finish(CommandFinish),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum CommandOutput {
    OutputLine(String),
    ErrorLine(String),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum CommandFinish {
    Success(bool),
    Error(String),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepCommand {
    pub inputs: StepCommandInputs,
    pub fail_if_missing: Option<VariableScalar>,
    pub selector: Option<Selector>,
    pub strategy: Option<VariableScalar>,
}

impl StepCommand {
    pub async fn run(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        workflow_run: &WorkflowRun,
    ) -> WorkflowResult<()> {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        let mut workflow_run_step =
            WorkflowRunStep::new(&txn, &nats, &workflow_run, &Step::Command(self.clone())).await?;

        // TODO: Only the linear strategy is implemented! eventually, this should
        // be some kind of wrapper around potential dispatch of this whole section.
        //
        // For now, it can just be a long ass adam special.
        for selection in workflow_run.ctx.selection.iter() {
            if workflow_run.ctx.inputs.is_none() {
                return Err(WorkflowError::NoInputs);
            }

            let inputs = workflow_run.ctx.inputs.as_ref().unwrap(); // Safe, we just checked it.

            let mut workflow_run_step_entity =
                WorkflowRunStepEntity::new(&txn, &nats, &selection.entity.id, &workflow_run_step)
                    .await?;

            // Command reqeust!!
            let request = CommandRequest {
                inputs,
                selection,
                system: workflow_run.ctx.system.as_ref(),
                workspace: &workflow_run.ctx.workspace,
            };

            let (progress_tx, mut progress_rx) =
                tokio::sync::mpsc::unbounded_channel::<CommandProtocol>();

            veritech
                .send_async("runCommand", request.clone(), progress_tx)
                .await?;

            while let Some(message) = progress_rx.recv().await {
                match message {
                    CommandProtocol::Start(_) => {
                        workflow_run_step_entity.state = WorkflowRunStepEntityState::Running;
                        workflow_run_step_entity.save(&txn, &nats).await?;
                    }
                    CommandProtocol::Finish(finish) => {
                        match finish {
                            CommandFinish::Success(_) => {
                                workflow_run_step_entity.state =
                                    WorkflowRunStepEntityState::Success;
                                let current_time = Utc::now();
                                let unix_timestamp = current_time.timestamp_millis();
                                let timestamp = format!("{}", current_time);
                                workflow_run_step_entity.end_timestamp = Some(timestamp);
                                workflow_run_step_entity.end_unix_timestamp = Some(unix_timestamp);
                                workflow_run_step_entity.save(&txn, &nats).await?;
                            }
                            CommandFinish::Error(error) => {
                                workflow_run_step.state = WorkflowRunStepState::Failure;
                                workflow_run_step_entity.state =
                                    WorkflowRunStepEntityState::Failure;
                                workflow_run_step_entity.error =
                                    Some(workflow_run_step_entity.error.as_mut().map_or_else(
                                        || error.clone(),
                                        |o| {
                                            o.push_str(&error);
                                            o.to_string()
                                        },
                                    ));
                                let current_time = Utc::now();
                                let unix_timestamp = current_time.timestamp_millis();
                                let timestamp = format!("{}", current_time);
                                workflow_run_step_entity.end_timestamp = Some(timestamp);
                                workflow_run_step_entity.end_unix_timestamp = Some(unix_timestamp);
                                workflow_run_step_entity.save(&txn, &nats).await?;
                            }
                        }
                        request
                            .selection
                            .resource
                            .clone()
                            .sync(pg.clone(), nats_conn.clone(), veritech.clone())
                            .await?;
                    }
                    CommandProtocol::Output(output) => match output {
                        CommandOutput::OutputLine(output) => {
                            workflow_run_step_entity.output =
                                Some(workflow_run_step_entity.output.as_mut().map_or_else(
                                    || output.clone(),
                                    |o| {
                                        o.push_str(&output);
                                        o.to_string()
                                    },
                                ));
                            workflow_run_step_entity.save(&txn, &nats).await?;
                        }
                        CommandOutput::ErrorLine(error) => {
                            workflow_run_step_entity.error =
                                Some(workflow_run_step_entity.error.as_mut().map_or_else(
                                    || error.clone(),
                                    |o| {
                                        o.push_str(&error);
                                        o.to_string()
                                    },
                                ));
                            workflow_run_step_entity.save(&txn, &nats).await?;
                        }
                    },
                }
            }
        }

        if workflow_run_step.state != WorkflowRunStepState::Failure {
            workflow_run_step.state = WorkflowRunStepState::Success;
        }
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);
        workflow_run_step.end_timestamp = Some(timestamp);
        workflow_run_step.end_unix_timestamp = Some(unix_timestamp);

        workflow_run_step.save(&txn, &nats).await?;
        txn.commit().await?;
        nats.commit().await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepActionInputs {
    name: VariableScalar,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepAction {
    pub inputs: StepActionInputs,
    pub fail_if_missing: Option<VariableScalar>,
    pub selector: Option<Selector>,
    pub strategy: Option<VariableScalar>,
}

impl StepAction {
    pub async fn run(
        &self,
        pg: &PgPool,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        workflow_run: &WorkflowRun,
    ) -> WorkflowResult<()> {
        let mut conn = pg.pool.get().await?;
        let txn = conn.transaction().await?;
        let nats = nats_conn.transaction();

        let mut workflow_run_step =
            WorkflowRunStep::new(&txn, &nats, &workflow_run, &Step::Action(self.clone())).await?;

        for selection in workflow_run.ctx.selection.iter() {
            if workflow_run.ctx.inputs.is_none() {
                return Err(WorkflowError::NoInputs);
            }
            let inputs = workflow_run.ctx.inputs.as_ref().unwrap(); // Safe, we just checked it
            let action_name = inputs["name"]
                .as_str()
                .ok_or(WorkflowError::NoNameInInputs)?;

            let mut workflow_run_step_entity =
                WorkflowRunStepEntity::new(&txn, &nats, &selection.entity.id, &workflow_run_step)
                    .await?;
            workflow_run_step_entity.state = WorkflowRunStepEntityState::Running;
            workflow_run_step_entity.output = Some(format!(
                "Running action {} on {} {}",
                &action_name, &selection.entity.entity_type, &selection.entity.name
            ));
            workflow_run_step_entity.save(&txn, &nats).await?;

            let workflow_name =
                Workflow::entity_and_action_name_to_workflow_name(&selection.entity, &action_name);

            let ctx = WorkflowContext {
                dry_run: workflow_run.ctx.dry_run.clone(),
                entity: Some(selection.entity.clone()),
                system: workflow_run.ctx.system.clone(),
                selection: vec![],
                strategy: None,
                fail_if_missing: None,
                inputs: None,
                args: None,
                output: None,
                store: None,
                workspace: workflow_run.ctx.workspace.clone(),
            };

            let workflow_run = Workflow::get_by_name(&txn, workflow_name)
                .await?
                .invoke_and_wait(&pg, &nats_conn, &veritech, ctx)
                .await?;
            // How to tell if we failed? Who the fuck nknows? I think we can check the state?
            workflow_run_step_entity.state = WorkflowRunStepEntityState::Success;
            let current_time = Utc::now();
            let unix_timestamp = current_time.timestamp_millis();
            let timestamp = format!("{}", current_time);
            workflow_run_step_entity.end_timestamp = Some(timestamp);
            workflow_run_step_entity.end_unix_timestamp = Some(unix_timestamp);
            workflow_run_step_entity.save(&txn, &nats).await?;
        }
        if workflow_run_step.state != WorkflowRunStepState::Failure {
            workflow_run_step.state = WorkflowRunStepState::Success;
        }
        let current_time = Utc::now();
        let unix_timestamp = current_time.timestamp_millis();
        let timestamp = format!("{}", current_time);
        workflow_run_step.end_timestamp = Some(timestamp);
        workflow_run_step.end_unix_timestamp = Some(unix_timestamp);

        workflow_run_step.save(&txn, &nats).await?;
        txn.commit().await?;
        nats.commit().await?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepWorkflowInputs {
    name: VariableScalar,
    args: Option<VariableArray>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StepWorkflow {
    pub inputs: StepWorkflowInputs,
    pub fail_if_missing: Option<VariableBool>,
    pub selector: Option<Selector>,
    pub strategy: Option<VariableScalar>,
}

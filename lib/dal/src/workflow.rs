use crate::{
    func::backend::js_workflow::FuncBackendJsWorkflowArgs, DalContext, Func, FuncBackendKind,
    FuncBinding, FuncBindingError, StandardModel, StandardModelError,
};
use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error("missing workflow {0}")]
    MissingWorkflow(String),
    #[error("missing command {0}")]
    MissingCommand(String),
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;

#[derive(
    Deserialize,
    Serialize,
    Debug,
    Display,
    AsRefStr,
    PartialEq,
    Eq,
    EnumIter,
    EnumString,
    Clone,
    Copy,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WorkflowKind {
    Conditional,
    Exceptional,
    Parallel,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum WorkflowStep {
    Workflow {
        workflow: String,
        #[serde(default)]
        args: Vec<serde_json::Value>,
    },
    Command {
        command: String,
        #[serde(default)]
        args: Vec<serde_json::Value>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WorkflowView {
    name: String,
    kind: WorkflowKind,
    steps: Vec<WorkflowStep>,
    args: Vec<serde_json::Value>,
}

impl WorkflowView {
    pub fn new(
        name: String,
        kind: WorkflowKind,
        steps: Vec<WorkflowStep>,
        args: Option<Vec<serde_json::Value>>,
    ) -> Self {
        Self {
            name,
            kind,
            steps,
            args: args.unwrap_or_default(),
        }
    }

    pub async fn resolve(ctx: &DalContext<'_, '_>, name: &str) -> WorkflowResult<WorkflowTree> {
        let args = vec![];
        Self::resolve_inner(ctx, name, args, HashSet::new(), &mut HashMap::new()).await
    }

    async fn veritech_run(
        ctx: &DalContext<'_, '_>,
        func: Func,
        args: FuncBackendJsWorkflowArgs,
    ) -> WorkflowResult<Self> {
        assert_eq!(func.backend_kind(), &FuncBackendKind::JsWorkflow);
        let (_func_binding, func_binding_return_value) =
            FuncBinding::find_or_create_and_execute(ctx, serde_json::to_value(args)?, *func.id())
                .await?;
        Ok(Self::deserialize(
            func_binding_return_value
                .value()
                .unwrap_or(&serde_json::Value::Null),
        )?)
    }

    #[async_recursion]
    async fn resolve_inner(
        ctx: &DalContext<'_, '_>,
        name: &str,
        _args: Vec<serde_json::Value>,
        mut recursion_marker: HashSet<String>,
        workflows_cache: &mut HashMap<String, WorkflowTree>,
    ) -> WorkflowResult<WorkflowTree> {
        recursion_marker.insert(name.to_owned());

        // TODO: name is not necessarily enough
        let func = Func::find_by_attr(ctx, "name", &name)
            .await?
            .pop()
            .ok_or_else(|| WorkflowError::MissingWorkflow(name.to_owned()))?;
        let view: WorkflowView = Self::veritech_run(ctx, func, FuncBackendJsWorkflowArgs).await?;

        let mut steps = Vec::with_capacity(view.steps.len());
        for step in view.steps {
            match step {
                WorkflowStep::Workflow { workflow, args } => {
                    if recursion_marker.contains(&workflow) {
                        panic!("Recursive workflow found: {}", workflow);
                    }

                    let key = format!("{workflow}-{}", serde_json::to_string(&args)?);
                    match workflows_cache.get(&key) {
                        Some(workflow) => steps.push(WorkflowTreeStep::Workflow(workflow.clone())),
                        None => {
                            let view = Self::resolve_inner(
                                ctx,
                                &workflow,
                                args,
                                recursion_marker.clone(),
                                workflows_cache,
                            )
                            .await?;

                            steps.push(WorkflowTreeStep::Workflow(view.clone()));
                            workflows_cache.insert(key, view);
                        }
                    }
                }
                WorkflowStep::Command { command, args } => {
                    // TODO: create builtins with the commands we want
                    //let func = Func::find_by_attr(ctx, "name", &command)
                    //    .await?
                    //    .pop()
                    //    .ok_or(WorkflowError::MissingCommand(command))?;
                    //assert_eq!(func.backend_kind(), &FuncBackendKind::JsCommand);
                    // TODO: cache this
                    steps.push(WorkflowTreeStep::Command {
                        func: command,
                        args: args.clone(),
                    })
                }
            }
        }
        Ok(WorkflowTree {
            name: view.name,
            kind: view.kind,
            steps,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum WorkflowTreeStep {
    Workflow(WorkflowTree),
    Command {
        func: String, //Func,
        #[serde(default)]
        args: Vec<serde_json::Value>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowTree {
    name: String,
    kind: WorkflowKind,
    steps: Vec<WorkflowTreeStep>,
}

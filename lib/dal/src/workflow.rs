use std::{collections::HashMap, collections::HashSet, sync::Arc};

use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;
use veritech_client::OutputStream;

use crate::{
    func::backend::js_workflow::FuncBackendJsWorkflowArgs, func::backend::FuncDispatchContext,
    func::binding::FuncBindingId, func::execution::FuncExecution, DalContext, DalContextBuilder,
    Func, FuncBackendKind, FuncBinding, FuncBindingError, FuncBindingReturnValue, PgPoolError,
    RequestContext, ServicesContext, StandardModel, StandardModelError, TransactionsError, WsEvent,
    WsEventError, WsEventResult, WsPayload,
};

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
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
    #[error("command not prepared {0}")]
    CommandNotPrepared(FuncBindingId),
    #[error("unset func binding {0}")]
    UnsetFuncBinding(FuncBindingId),
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
        args: serde_json::Value,
    },
    Command {
        command: String,
        #[serde(default)]
        args: serde_json::Value,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WorkflowView {
    name: String,
    kind: WorkflowKind,
    steps: Vec<WorkflowStep>,
    args: serde_json::Value,
}

impl WorkflowView {
    pub fn new(
        name: String,
        kind: WorkflowKind,
        steps: Vec<WorkflowStep>,
        args: serde_json::Value,
    ) -> Self {
        Self {
            name,
            kind,
            steps,
            args,
        }
    }

    pub async fn resolve(
        ctx: &DalContext,
        func: &Func,
        args: serde_json::Value,
    ) -> WorkflowResult<WorkflowTree> {
        Self::resolve_inner(ctx, func.name(), args, HashSet::new(), &mut HashMap::new()).await
    }

    /// Run a workflow using veritech.
    async fn veritech_run(
        ctx: &DalContext,
        func: Func,
        args: FuncBackendJsWorkflowArgs,
    ) -> WorkflowResult<Self> {
        assert_eq!(func.backend_kind(), &FuncBackendKind::JsWorkflow);
        let func_binding = FuncBinding::new(
            ctx,
            serde_json::to_value(&args)?,
            *func.id(),
            *func.backend_kind(),
        )
        .await?;
        let func_binding_return_value = func_binding.execute(ctx).await?;
        Ok(Self::deserialize(
            func_binding_return_value
                .value()
                .ok_or_else(|| WorkflowError::UnsetFuncBinding(*func_binding.id()))?,
        )?)
    }

    /// A recursive function to resolve inner workflows in order to build a [`WorkflowTree`].
    #[async_recursion]
    async fn resolve_inner(
        ctx: &DalContext,
        name: &str,
        args: FuncBackendJsWorkflowArgs,
        mut recursion_marker: HashSet<String>,
        workflows_cache: &mut HashMap<String, WorkflowTree>,
    ) -> WorkflowResult<WorkflowTree> {
        recursion_marker.insert(name.to_owned());

        // TODO: name is not necessarily enough
        let func = Func::find_by_attr(ctx, "name", &name)
            .await?
            .pop()
            .ok_or_else(|| WorkflowError::MissingWorkflow(name.to_owned()))?;
        let view = Self::veritech_run(ctx, func, args).await?;

        let mut steps = Vec::with_capacity(view.steps.len());
        for step in view.steps {
            match step {
                WorkflowStep::Workflow { workflow, args } => {
                    if recursion_marker.contains(&workflow) {
                        panic!("Recursive workflow found: {workflow}");
                    }

                    let key = format!("{workflow}-{}", serde_json::to_string(&args)?);
                    match workflows_cache.get(&key) {
                        Some(workflow) => steps.push(WorkflowTreeStep::Workflow(workflow.clone())),
                        None => {
                            let tree = Self::resolve_inner(
                                ctx,
                                &workflow,
                                args,
                                recursion_marker.clone(),
                                workflows_cache,
                            )
                            .await?;

                            steps.push(WorkflowTreeStep::Workflow(tree.clone()));
                            workflows_cache.insert(key, tree);
                        }
                    }
                }
                WorkflowStep::Command { command, args } => {
                    let func = Func::find_by_attr(ctx, "name", &command)
                        .await?
                        .pop()
                        .ok_or(WorkflowError::MissingCommand(command))?;
                    assert_eq!(func.backend_kind(), &FuncBackendKind::JsCommand);
                    let func_binding = FuncBinding::new(
                        ctx,
                        serde_json::to_value(args)?,
                        *func.id(),
                        *func.backend_kind(),
                    )
                    .await?;
                    // TODO: cache this
                    steps.push(WorkflowTreeStep::Command { func_binding })
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
    Command { func_binding: FuncBinding },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkflowTree {
    pub name: String,
    pub kind: WorkflowKind,
    pub steps: Vec<WorkflowTreeStep>,
}

#[derive(Debug, Clone)]
pub struct FuncToExecute {
    index: usize,
    func_binding: FuncBinding,
    func: Func,
    execution: FuncExecution,
    context: FuncDispatchContext,
    value: (Option<serde_json::Value>, Option<serde_json::Value>),
}

impl WorkflowTree {
    pub async fn run(
        &self,
        ctx: &DalContext,
        run_id: usize,
    ) -> WorkflowResult<Vec<FuncBindingReturnValue>> {
        let (map, rxs) = self.prepare(ctx).await?;

        let mut handlers = tokio::task::JoinSet::new();
        for (func_binding_id, rx) in rxs {
            let services_context = ServicesContext::new(
                ctx.pg_pool().clone(),
                ctx.nats_conn().clone(),
                ctx.job_processor(),
                ctx.veritech().clone(),
                Arc::new(*ctx.encryption_key()),
                None,
            );
            let ctx_builder = services_context.clone().into_builder();
            let request_context = RequestContext {
                tenancy: *ctx.tenancy(),
                visibility: *ctx.visibility(),
                history_actor: *ctx.history_actor(),
            };

            handlers.spawn(process_output(
                ctx_builder,
                request_context,
                func_binding_id,
                rx,
                run_id,
            ));
        }
        let mut map = self.clone().execute(map).await?;

        for prepared in map.values_mut() {
            // Drops tx so rx will stop waiting for it
            let (mut tx, _) = mpsc::channel(1);
            std::mem::swap(&mut prepared.context.output_tx, &mut tx);
        }

        let mut outputs = HashMap::new();
        while let Some(res) = handlers.join_next().await {
            let (func_binding_id, output) = join_task(res)?;
            outputs.insert(func_binding_id, output);
        }
        self.postprocess(ctx, map, outputs).await
    }

    #[async_recursion]
    async fn prepare(
        &self,
        ctx: &DalContext,
    ) -> WorkflowResult<(
        HashMap<FuncBindingId, FuncToExecute>,
        HashMap<FuncBindingId, mpsc::Receiver<OutputStream>>,
    )> {
        let mut map = HashMap::new();
        let mut rxs = HashMap::new();
        let mut index = 0;
        for step in &self.steps {
            match step {
                WorkflowTreeStep::Command { func_binding } => {
                    index += 1;
                    let id = *func_binding.id();
                    let func_binding = func_binding.clone();
                    let (func, execution, context, rx) =
                        func_binding.prepare_execution(ctx).await?;
                    map.insert(
                        id,
                        FuncToExecute {
                            index,
                            func_binding,
                            func,
                            execution,
                            context,
                            value: (None, None),
                        },
                    );
                    rxs.insert(id, rx);
                }
                WorkflowTreeStep::Workflow(workflow) => {
                    let (m, r) = workflow.prepare(ctx).await?;
                    let count = m.len();
                    map.extend(m.into_iter().map(|(id, mut meta)| {
                        meta.index += index;
                        (id, meta)
                    }));
                    index += count;
                    rxs.extend(r);
                }
            }
        }
        Ok((map, rxs))
    }

    // Note: too damn many clones
    #[async_recursion]
    async fn execute(
        self,
        mut map: HashMap<FuncBindingId, FuncToExecute>,
    ) -> WorkflowResult<HashMap<FuncBindingId, FuncToExecute>> {
        match self.kind {
            WorkflowKind::Conditional => {
                for step in self.steps {
                    match step {
                        WorkflowTreeStep::Command { func_binding } => {
                            let mut prepared = map.get_mut(func_binding.id()).ok_or_else(|| {
                                WorkflowError::CommandNotPrepared(*func_binding.id())
                            })?;
                            prepared.value = func_binding
                                .execute_critical_section(
                                    prepared.func.clone(),
                                    prepared.context.clone(),
                                )
                                .await?;
                        }
                        WorkflowTreeStep::Workflow(workflow) => {
                            map.extend(workflow.clone().execute(map.clone()).await?)
                        }
                    }
                }
            }
            WorkflowKind::Parallel => {
                let mut commands = tokio::task::JoinSet::new();
                let mut workflows = tokio::task::JoinSet::new();
                for step in self.steps {
                    match step {
                        WorkflowTreeStep::Command { func_binding } => {
                            let func_binding = func_binding.clone();
                            let prepared = map.get(func_binding.id()).ok_or_else(|| {
                                WorkflowError::CommandNotPrepared(*func_binding.id())
                            })?;
                            let (func, context) = (prepared.func.clone(), prepared.context.clone());
                            commands.spawn(async move {
                                func_binding
                                    .clone()
                                    .execute_critical_section(func, context)
                                    .await
                                    .map(|value| (func_binding, value))
                            });
                        }
                        WorkflowTreeStep::Workflow(workflow) => {
                            let map = map.clone();
                            workflows.spawn(async move { workflow.execute(map).await });
                        }
                    }
                }

                // TODO: poll both in the same future

                while let Some(res) = commands.join_next().await {
                    let (func_binding, value) = join_task(res)?;
                    let mut prepared = map.get_mut(func_binding.id()).ok_or_else(move || {
                        WorkflowError::CommandNotPrepared(*func_binding.id())
                    })?;
                    prepared.value = value;
                }

                while let Some(res) = workflows.join_next().await {
                    map.extend(join_task(res)?);
                }
            }
            WorkflowKind::Exceptional => todo!(),
        }
        Ok(map)
    }

    async fn postprocess(
        &self,
        ctx: &DalContext,
        map: HashMap<FuncBindingId, FuncToExecute>,
        mut outputs: HashMap<FuncBindingId, Vec<OutputStream>>,
    ) -> WorkflowResult<Vec<FuncBindingReturnValue>> {
        let mut values = Vec::with_capacity(map.len());
        // Do we have a problem here, if the same func_binding gets executed twice?
        for (_, prepared) in map {
            let id = *prepared.func_binding.id();
            let output = outputs
                .remove(&id)
                .ok_or(WorkflowError::CommandNotPrepared(id))?;
            let func_binding_return_value = prepared
                .func_binding
                .postprocess_execution(
                    ctx,
                    output,
                    &prepared.func,
                    prepared.value,
                    prepared.execution,
                )
                .await?;
            values.push((prepared.index, func_binding_return_value));
        }
        values.sort_by_key(|(index, _)| *index);
        Ok(values.into_iter().map(|(_, v)| v).collect())
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged, rename_all = "camelCase")]
pub enum WorkflowTreeStepView {
    Workflow(WorkflowTreeView),
    Command {
        command: String,
        args: serde_json::Value,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTreeView {
    name: String,
    kind: WorkflowKind,
    steps: Vec<WorkflowTreeStepView>,
}

impl WorkflowTreeView {
    // We need to stop recursing so much
    #[async_recursion]
    pub async fn new(ctx: &DalContext, tree: WorkflowTree) -> WorkflowResult<Self> {
        let mut view = WorkflowTreeView {
            name: tree.name,
            kind: tree.kind,
            steps: Vec::with_capacity(tree.steps.len()),
        };
        for step in tree.steps {
            match step {
                WorkflowTreeStep::Command { func_binding } => {
                    view.steps.push(WorkflowTreeStepView::Command {
                        command: func_binding
                            .func(ctx)
                            .await?
                            .ok_or_else(|| FuncBindingError::FuncNotFound(*func_binding.pk()))?
                            .name()
                            .to_owned(),
                        args: func_binding.args().clone(),
                    })
                }
                WorkflowTreeStep::Workflow(tree) => view
                    .steps
                    .push(WorkflowTreeStepView::Workflow(Self::new(ctx, tree).await?)),
            }
        }
        Ok(view)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandOutput {
    run_id: usize,
    output: String,
}

impl WsEvent {
    pub async fn command_output(
        ctx: &DalContext,
        run_id: usize,
        output: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::CommandOutput(CommandOutput { run_id, output }),
        )
        .await
    }
}

fn join_task<T>(res: Result<T, tokio::task::JoinError>) -> T {
    match res {
        Ok(t) => t,
        Err(err) => {
            assert!(!err.is_cancelled(), "Task got cancelled but shouldn't");
            let any = err.into_panic();
            // Note: Technically panics can be of any form, but most should be &str or String
            match any.downcast::<String>() {
                Ok(msg) => panic!("{}", msg),
                Err(any) => match any.downcast::<&str>() {
                    Ok(msg) => panic!("{}", msg),
                    Err(any) => panic!("Panic message downcast failed of {:?}", any.type_id()),
                },
            }
        }
    }
}

async fn process_output(
    ctx_builder: DalContextBuilder,
    request_context: RequestContext,
    func_binding_id: FuncBindingId,
    mut rx: mpsc::Receiver<OutputStream>,
    run_id: usize,
) -> WorkflowResult<(FuncBindingId, Vec<OutputStream>)> {
    let ctx = ctx_builder.build(request_context).await?;

    let mut output = Vec::new();
    while let Some(stream) = rx.recv().await {
        let text = stream.message.clone();
        output.push(stream);

        WsEvent::command_output(&ctx, run_id, text)
            .await?
            .publish_on_commit(&ctx)
            .await?;
        ctx.commit().await?;
    }

    Ok((func_binding_id, output))
}

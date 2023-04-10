use dal::{DalContext, Func, StandardModel, WorkflowKind, WorkflowTreeStep, WorkflowView};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

#[test]
async fn resolve(ctx: &DalContext) {
    let name = "si:poemWorkflow";
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {name}"));
    let mut tree = WorkflowView::resolve(
        ctx,
        &func,
        serde_json::Value::String("Domingos Passos".to_owned()),
    )
    .await
    .expect("unable to resolve workflow");

    assert_eq!("si:poemWorkflow", tree.name);
    assert_eq!(WorkflowKind::Conditional, tree.kind);
    assert_eq!(8, tree.steps.len());

    let mut step_iter = tree.steps.drain(0..);

    // Step 1
    match step_iter.next() {
        Some(WorkflowTreeStep::Workflow(_)) => {
            panic!("Expected workflow step to be of kind Command")
        }
        Some(WorkflowTreeStep::Command { func_binding }) => {
            let step_func = Func::for_binding(ctx, &func_binding)
                .await
                .expect("Could not retrieve Func for FuncBinding");
            assert_eq!("si:leroLeroStanza1Command", step_func.name());
            assert_eq!(serde_json::Value::Null, *func_binding.args());
        }
        None => panic!("Unable to examine workflow step"),
    }

    // Step 2
    match step_iter.next() {
        Some(WorkflowTreeStep::Workflow(_)) => {
            panic!("Expected workflow step to be of kind Command")
        }
        Some(WorkflowTreeStep::Command { func_binding }) => {
            let step_func = Func::for_binding(ctx, &func_binding)
                .await
                .expect("Could not retrieve Func for FuncBinding");
            assert_eq!("si:leroLeroStanza2Command", step_func.name());
            assert_eq!(serde_json::Value::Null, *func_binding.args());
        }
        None => panic!("Unable to examine workflow step"),
    }

    // Step 3
    match step_iter.next() {
        Some(WorkflowTreeStep::Workflow(_)) => {
            panic!("Expected workflow step to be of kind Command")
        }
        Some(WorkflowTreeStep::Command { func_binding }) => {
            let step_func = Func::for_binding(ctx, &func_binding)
                .await
                .expect("Could not retrieve Func for FuncBinding");
            assert_eq!("si:leroLeroStanza3Command", step_func.name());
            assert_eq!(serde_json::Value::Null, *func_binding.args());
        }
        None => panic!("Unable to examine workflow step"),
    }

    // Step 4
    match step_iter.next() {
        Some(WorkflowTreeStep::Workflow(_)) => {
            panic!("Expected workflow step to be of kind Command")
        }
        Some(WorkflowTreeStep::Command { func_binding }) => {
            let step_func = Func::for_binding(ctx, &func_binding)
                .await
                .expect("Could not retrieve Func for FuncBinding");
            assert_eq!("si:leroLeroStanza4Command", step_func.name());
            assert_eq!(serde_json::Value::Null, *func_binding.args());
        }
        None => panic!("Unable to examine workflow step"),
    }

    // Step 5
    match step_iter.next() {
        Some(WorkflowTreeStep::Workflow(_)) => {
            panic!("Expected workflow step to be of kind Command")
        }
        Some(WorkflowTreeStep::Command { func_binding }) => {
            let step_func = Func::for_binding(ctx, &func_binding)
                .await
                .expect("Could not retrieve Func for FuncBinding");
            assert_eq!("si:leroLeroStanza5Command", step_func.name());
            assert_eq!(serde_json::Value::Null, *func_binding.args());
        }
        None => panic!("Unable to examine workflow step"),
    }

    // Step 6
    match step_iter.next() {
        Some(WorkflowTreeStep::Workflow(_)) => {
            panic!("Expected workflow step to be of kind Command")
        }
        Some(WorkflowTreeStep::Command { func_binding }) => {
            let step_func = Func::for_binding(ctx, &func_binding)
                .await
                .expect("Could not retrieve Func for FuncBinding");
            assert_eq!("si:leroLeroStanza6Command", step_func.name());
            assert_eq!(serde_json::Value::Null, *func_binding.args());
        }
        None => panic!("Unable to examine workflow step"),
    }

    // Step 7
    match step_iter.next() {
        Some(WorkflowTreeStep::Workflow(_)) => {
            panic!("Expected workflow step to be of kind Command")
        }
        Some(WorkflowTreeStep::Command { func_binding }) => {
            let step_func = Func::for_binding(ctx, &func_binding)
                .await
                .expect("Could not retrieve Func for FuncBinding");
            assert_eq!("si:leroLeroStanza7Command", step_func.name());
            assert_eq!(serde_json::Value::Null, *func_binding.args());
        }
        None => panic!("Unable to examine workflow step"),
    }

    // Step 8
    match step_iter.next() {
        Some(WorkflowTreeStep::Workflow(mut step_workflow)) => {
            assert_eq!("si:finalizingWorkflow", step_workflow.name);
            assert_eq!(WorkflowKind::Parallel, step_workflow.kind);
            assert_eq!(2, step_workflow.steps.len());

            let mut substep_iter = step_workflow.steps.drain(0..);

            // Sub-Step 1
            match substep_iter.next() {
                Some(WorkflowTreeStep::Workflow(_)) => {
                    panic!("Expected sub-workflow step to be of kind Command");
                }
                Some(WorkflowTreeStep::Command { func_binding }) => {
                    let step_func = Func::for_binding(ctx, &func_binding)
                        .await
                        .expect("Could not retrieve Func for FuncBinding");
                    assert_eq!("si:leroLeroQuestionCommand", step_func.name());
                    assert_eq!(json!(["Domingos Passos".to_owned()]), *func_binding.args());
                }
                None => panic!("Unable to examine workflow step"),
            }

            // Sub-Step 2
            match substep_iter.next() {
                Some(WorkflowTreeStep::Workflow(_)) => {
                    panic!("Expected sub-workflow step to be of kind Command");
                }
                Some(WorkflowTreeStep::Command { func_binding }) => {
                    let step_func = Func::for_binding(ctx, &func_binding)
                        .await
                        .expect("Could not retrieve Func for FuncBinding");
                    assert_eq!("si:leroLeroByeCommand", step_func.name());
                    assert_eq!(serde_json::Value::Null, *func_binding.args());
                }
                None => panic!("Unable to examine workflow step"),
            }

            // No more sub-steps!
            assert_eq!(None, substep_iter.next());
        }
        Some(WorkflowTreeStep::Command { func_binding: _ }) => {
            panic!("Expected workflow step to be of kind Workflow")
        }
        None => panic!("Unable to examine workflow step"),
    }

    // No more steps!
    assert_eq!(None, step_iter.next());
}

#[test]
async fn run(ctx: DalContext) {
    let name = "si:poemWorkflow";
    let func = Func::find_by_attr(&ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {name}"));
    let tree = WorkflowView::resolve(
        &ctx,
        &func,
        serde_json::Value::String("Domingos Passos".to_owned()),
    )
    .await
    .expect("unable to resolve workflow");

    // Needed as workflow run create new transactions
    ctx.blocking_commit()
        .await
        .expect("unable to commit transaction");

    // Text output is checked at WorkflowRunner tests as they actually order it
    tree.run(&ctx, 0).await.expect("unable to run workflow");
}

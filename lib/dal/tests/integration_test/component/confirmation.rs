use dal::action_prototype::ActionKind;
use dal::component::confirmation::view::RecommendationStatus;
use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::func::backend::js_command::CommandRunResult;
use dal::job::definition::{FixItem, FixesJob};
use dal::schema::variant::leaves::LeafKind;
use dal::{
    component::confirmation::view::ConfirmationStatus,
    generate_name,
    schema::variant::leaves::{LeafInput, LeafInputLocation},
    ActionPrototype, ActionPrototypeContext, ChangeSet, ChangeSetStatus, Component, ComponentView,
    ComponentViewProperties, DalContext, Fix, FixBatch, FixCompletionStatus, Func, FuncBackendKind,
    FuncBackendResponseType, SchemaVariant, StandardModel, Visibility, WorkflowPrototype,
    WorkflowPrototypeContext,
};
use dal_test::test;
use dal_test::test_harness::{create_schema, create_schema_variant_with_root};
use pretty_assertions_sorted::assert_eq;
use veritech_client::ResourceStatus;

/// Recommendation: run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[test]
async fn add_and_run_confirmations(mut octx: DalContext) {
    let ctx = &mut octx;
    ctx.update_to_head();

    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, _root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    let schema_variant_id = *schema_variant.id();
    schema
        .set_default_schema_variant_id(ctx, Some(schema_variant_id))
        .await
        .expect("cannot set default schema variant");

    // Create command and workflow funcs for our workflow and action prototypes.
    let mut command_func = Func::new(
        ctx,
        "test:createCommand",
        FuncBackendKind::JsCommand,
        FuncBackendResponseType::Command,
    )
    .await
    .expect("could not create func");
    let code = "async function create() {
      return { value: \"poop\", status: \"ok\" };
    }";
    command_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    command_func
        .set_handler(ctx, Some("create"))
        .await
        .expect("set handler");
    let mut workflow_func = Func::new(
        ctx,
        "test:createWorkflow",
        FuncBackendKind::JsWorkflow,
        FuncBackendResponseType::Workflow,
    )
    .await
    .expect("could not create func");
    let code = "async function create() {
      return {
        name: \"test:createWorkflow\",
        kind: \"conditional\",
        steps: [
          {
            command: \"test:createCommand\",
          },
        ],
      };
    }";
    workflow_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    workflow_func
        .set_handler(ctx, Some("create"))
        .await
        .expect("set handler");

    // Create workflow and action protoypes.
    let workflow_prototype = WorkflowPrototype::new(
        ctx,
        *workflow_func.id(),
        serde_json::Value::Null,
        WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        },
        "create",
    )
    .await
    .expect("could not create workflow prototype");
    ActionPrototype::new(
        ctx,
        *workflow_prototype.id(),
        "create",
        ActionKind::Create,
        ActionPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id,
            ..Default::default()
        },
    )
    .await
    .expect("unable to create action prototype");

    // Setup the confirmation function.
    let mut confirmation_func = Func::new(
        ctx,
        "test:confirmation",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Confirmation,
    )
    .await
    .expect("could not create func");
    let confirmation_func_id = *confirmation_func.id();
    let code = "async function exists(input) {
        if (!input.resource?.value) {
            return {
                success: false,
                recommendedActions: [\"create\"]
            }
        }
        return {
            success: true,
            recommendedActions: [],
        }
    }";
    confirmation_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    confirmation_func
        .set_handler(ctx, Some("exists"))
        .await
        .expect("set handler");
    let confirmation_func_argument = FuncArgument::new(
        ctx,
        "resource",
        FuncArgumentKind::String,
        None,
        confirmation_func_id,
    )
    .await
    .expect("could not create func argument");

    // Add the leaf for the confirmation.
    SchemaVariant::add_leaf(
        ctx,
        confirmation_func_id,
        schema_variant_id,
        None,
        LeafKind::Confirmation,
        vec![LeafInput {
            location: LeafInputLocation::Resource,
            func_argument_id: *confirmation_func_argument.id(),
        }],
    )
    .await
    .expect("could not add qualification");

    // Finalize the schema variant and create the component.
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let new_change_set = ChangeSet::new(ctx, generate_name(), None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(new_change_set.pk, None));

    // Create a component and immediately apply the change set.
    let (component, _) = Component::new(ctx, "component", schema_variant_id)
        .await
        .expect("cannot create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");
    assert_eq!(new_change_set.pk, ctx.visibility().change_set_pk);
    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("could not fetch change set by pk")
        .expect("no change set found for pk");
    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);
    ctx.update_visibility(Visibility::new_head(false));

    // Observe that the confirmation failed.
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "domain": {},
            "confirmation": {
                "test:confirmation": {
                    "success": false,
                    "recommendedActions": ["create"]
                }
            }
        }], // expected
        component_view.properties // actual
    );

    // "Create" the resource.
    component
        .set_resource(
            ctx,
            CommandRunResult {
                status: ResourceStatus::Ok,
                value: Some(serde_json::json!["poop"]),
                message: None,
                logs: vec![],
                last_synced: Default::default(),
            },
            true,
        )
        .await
        .expect("could not set resource");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe that the confirmation worked after "creation".
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "domain": {},
            "resource": {
                "logs": [],
                "value": "poop",
                "status": "ok"
            },
            "confirmation": {
                "test:confirmation": {
                    "success": true,
                    "recommendedActions": []
                }
            }
        }], // expected
        component_view.properties // actual
    );

    // "Delete" the resource.
    component
        .set_resource(
            ctx,
            CommandRunResult {
                status: ResourceStatus::Ok,
                value: None,
                message: None,
                logs: vec![],
                last_synced: Default::default(),
            },
            true,
        )
        .await
        .expect("could not set resource");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe that the confirmation worked after "deletion".
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "domain": {},
            "resource": {
                "logs": [],
                "status": "ok"
            },
            "confirmation": {
                "test:confirmation": {
                    "success": false,
                    "recommendedActions": ["create"]
                }
            }
        }], // expected
        component_view.properties // actual
    );
}

/// Recommendation: run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[test]
async fn list_confirmations(mut octx: DalContext) {
    let ctx = &mut octx;
    ctx.update_to_head();

    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, _root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    let schema_variant_id = *schema_variant.id();
    schema
        .set_default_schema_variant_id(ctx, Some(schema_variant_id))
        .await
        .expect("cannot set default schema variant");

    // Setup the confirmation function.
    let mut confirmation_func = Func::new(
        ctx,
        "test:confirmation",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Confirmation,
    )
    .await
    .expect("could not create func");
    let confirmation_func_id = *confirmation_func.id();
    let code = "async function exists(input) {
        if (!input.resource?.value) {
            return {
                success: false,
                recommendedActions: [\"create\"]
            }
        }
        return {
            success: true,
            recommendedActions: [],
        }
    }";
    confirmation_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    confirmation_func
        .set_handler(ctx, Some("exists"))
        .await
        .expect("set handler");
    let confirmation_func_argument = FuncArgument::new(
        ctx,
        "resource",
        FuncArgumentKind::String,
        None,
        confirmation_func_id,
    )
    .await
    .expect("could not create func argument");

    // Add the leaf for the confirmation.
    SchemaVariant::add_leaf(
        ctx,
        confirmation_func_id,
        schema_variant_id,
        None,
        LeafKind::Confirmation,
        vec![LeafInput {
            location: LeafInputLocation::Resource,
            func_argument_id: *confirmation_func_argument.id(),
        }],
    )
    .await
    .expect("could not add qualification");

    // Create command and workflow funcs for our workflow and action prototypes.
    let mut command_func = Func::new(
        ctx,
        "test:createCommand",
        FuncBackendKind::JsCommand,
        FuncBackendResponseType::Command,
    )
    .await
    .expect("could not create func");
    let code = "async function create() {
      return { value: \"poop\", status: \"ok\" };
    }";
    command_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    command_func
        .set_handler(ctx, Some("create"))
        .await
        .expect("set handler");
    let mut workflow_func = Func::new(
        ctx,
        "test:createWorkflow",
        FuncBackendKind::JsWorkflow,
        FuncBackendResponseType::Workflow,
    )
    .await
    .expect("could not create func");
    let code = "async function create() {
      return {
        name: \"test:createWorkflow\",
        kind: \"conditional\",
        steps: [
          {
            command: \"test:createCommand\",
          },
        ],
      };
    }";
    workflow_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    workflow_func
        .set_handler(ctx, Some("create"))
        .await
        .expect("set handler");

    // Create workflow and action prototypes.
    let workflow_prototype = WorkflowPrototype::new(
        ctx,
        *workflow_func.id(),
        serde_json::Value::Null,
        WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        },
        "create",
    )
    .await
    .expect("could not create workflow prototype");
    ActionPrototype::new(
        ctx,
        *workflow_prototype.id(),
        "create",
        ActionKind::Create,
        ActionPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id,
            ..Default::default()
        },
    )
    .await
    .expect("unable to create action prototype");

    // Finalize the schema variant and create the component.
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let new_change_set = ChangeSet::new(ctx, generate_name(), None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(new_change_set.pk, None));

    // Create a component and immediately apply the change set.
    let (component, _) = Component::new(ctx, "component", schema_variant_id)
        .await
        .expect("cannot create component");

    assert_eq!(new_change_set.pk, ctx.visibility().change_set_pk);
    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("could not fetch change set by pk")
        .expect("no change set found for pk");
    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);
    ctx.update_visibility(Visibility::new_head(false));

    // List confirmations.
    let (confirmations, mut recommendations) = Component::list_confirmations(ctx)
        .await
        .expect("could not list confirmations");
    assert_eq!(
        1,                   // expected
        confirmations.len()  // actual
    );
    let recommendation = recommendations.pop().expect("recommendations are empty");

    // Check that there is only one recommendation and that it looks as expected.
    assert!(recommendations.is_empty());
    assert_eq!(
        "create",                           // expected
        &recommendation.recommended_action  // actual
    );
    assert_eq!(
        None,                    // expected
        recommendation.last_fix  // actual
    );

    // Observe that the confirmation "failed" (i.e. did not fail execution, but returned
    // with an unsuccessful result).
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "domain": {},
            "confirmation": {
                "test:confirmation": {
                    "success": false,
                    "recommendedActions": ["create"]
                }
            }
        }], // expected
        component_view.properties // actual
    );

    // Run the fix from our recommendation.
    let batch = FixBatch::new(ctx, "toddhoward@systeminit.com")
        .await
        .expect("could not create fix batch");
    let fix = Fix::new(
        ctx,
        *batch.id(),
        recommendation.confirmation_attribute_value_id,
        recommendation.component_id,
        &recommendation.recommended_action,
    )
    .await
    .expect("could not create fix");
    let fixes = vec![FixItem {
        id: *fix.id(),
        attribute_value_id: recommendation.confirmation_attribute_value_id,
        component_id: recommendation.component_id,
        action: recommendation.recommended_action,
    }];
    ctx.enqueue_job(FixesJob::new(ctx, fixes, *batch.id()))
        .await
        .expect("failed to enqueue job");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure that our confirmations views look as intended. We should have exactly zero
    // recommendations!
    let (mut confirmations, recommendations) = Component::list_confirmations(ctx)
        .await
        .expect("could not list confirmations");
    let confirmation = confirmations.pop().expect("views are empty");
    assert!(confirmations.is_empty());
    assert_eq!(confirmation.status, ConfirmationStatus::Success);
    assert!(recommendations.is_empty());

    // Observe that the confirmation worked after "creation".
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "domain": {},
            "resource": {
                "logs": [],
                "value": "poop",
                "status": "ok"
            },
            "confirmation": {
                "test:confirmation": {
                    "success": true,
                    "recommendedActions": []
                }
            }
        }], // expected
        ComponentViewProperties::try_from(component_view)
            .expect("could not create component view properties")
            .drop_resource_last_synced()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // "Delete" the resource. This is similar to deleting the resource in the "real world" and
    // waiting for a resource sync, but the model has remained the same.
    component
        .set_resource(
            ctx,
            CommandRunResult {
                status: ResourceStatus::Ok,
                value: None,
                message: None,
                logs: vec![],
                last_synced: Default::default(),
            },
            true,
        )
        .await
        .expect("could not set resource");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // List confirmations.
    let (confirmations, mut recommendations) = Component::list_confirmations(ctx)
        .await
        .expect("could not list confirmations");
    assert_eq!(
        1,                   // expected
        confirmations.len()  // actual
    );
    let recommendation = recommendations.pop().expect("recommendations are empty");

    // Check that there is only one recommendation and that it looks as expected.
    assert!(recommendations.is_empty());
    assert_eq!(
        "create",                           // expected
        &recommendation.recommended_action  // actual
    );
    assert_eq!(
        RecommendationStatus::Success, // expected
        recommendation.status          // actual
    );
    assert_eq!(
        FixCompletionStatus::Success, // expected
        recommendation
            .last_fix
            .expect("last fix not found")
            .status()  // actual
    );

    // Observe that the confirmation worked after "deletion". The component should be re-creatable.
    // The "success" field should be "false" because we have at least one recommended action (in
    // this case, it should be exactly one).
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "domain": {},
            "resource": {
                "logs": [],
                "status": "ok"
            },
            "confirmation": {
                "test:confirmation": {
                    "success": false,
                    "recommendedActions": ["create"]
                }
            }
        }], // expected
        component_view.properties // actual
    );
}

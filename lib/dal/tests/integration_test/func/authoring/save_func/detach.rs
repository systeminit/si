use dal::action::prototype::{ActionKind, ActionPrototype};
use dal::action::{Action, ActionId};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::summary::FuncSummary;
use dal::func::view::FuncView;
use dal::func::{AttributePrototypeBag, FuncAssociations};
use dal::{DalContext, Func, Schema, SchemaVariant};
use dal_test::helpers::{create_component_for_schema_name, ChangeSetTestHelpers};
use dal_test::test;

#[test]
async fn detach_attribute_func(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("unable to find by name")
        .expect("no schema found");
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");

    // Cache the total number of funcs before continuing.
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    let total_funcs = funcs.len();

    // Detach one action func to the schema variant and commit.
    let func_id = Func::find_id_by_name(ctx, "test:falloutEntriesToGalaxies")
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("unable to get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("unable to assemble a func view");
    let prototypes = func_view
        .associations
        .expect("empty associations")
        .get_attribute_internals()
        .expect("could not get internals");
    let prototype: AttributePrototypeBag = prototypes
        .into_iter()
        .find(|p| p.schema_variant_id == Some(schema_variant_id))
        .expect("has a prototype for this schema variant");

    FuncAuthoringClient::remove_attribute_prototype(ctx, prototype.id)
        .await
        .expect("could not remove attribute prototype");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Now, let's list all funcs and see what's left.
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    assert_eq!(
        total_funcs - 1, // expected
        funcs.len()      // actual
    );
    assert!(!funcs.iter().any(|summary| summary.id == func_id));
}

#[test]
async fn detach_attach_then_delete_action_func_while_enqueued(ctx: &mut DalContext) {
    pub async fn can_assemble(ctx: &DalContext, action_id: ActionId) -> bool {
        let action = Action::get_by_id(ctx, action_id)
            .await
            .expect("unable to get action");
        let prototype_id = Action::prototype_id(ctx, action_id)
            .await
            .expect("unable to get prototype id");
        let _prototype = ActionPrototype::get_by_id(ctx, prototype_id)
            .await
            .expect("unable to get prototype");
        let _func_run_id = ctx
            .layer_db()
            .func_run()
            .get_last_run_for_action_id(ctx.events_tenancy().workspace_pk, action.id().into())
            .await
            .expect("unable to get func run id")
            .map(|f| f.id());
        let _component_id = Action::component_id(ctx, action_id)
            .await
            .expect("unable to get component id");
        let _my_dependencies = action
            .get_all_dependencies(ctx)
            .await
            .expect("unable to get dependencies");
        let _dependent_on = Action::get_dependent_actions_by_id(ctx, action_id)
            .await
            .expect("unable to get dependent actions");
        let _hold_status_influenced_by = action
            .get_hold_status_influenced_by(ctx)
            .await
            .expect("unable to get hold status");
        true
    }
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("unable to find by name")
        .expect("no schema found");
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");

    // Cache the total number of funcs before continuing.
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    let initial_total_funcs = funcs.len();

    // create a component
    let new_component = create_component_for_schema_name(ctx, "starfield", "component")
        .await
        .expect("unable to create new component");

    // check that the action func has been enqueued
    let enqueued_actions = Action::list_topologically(ctx)
        .await
        .expect("can list actions");
    // create action views

    let mut queued = Vec::new();

    for action_id in enqueued_actions.into_iter() {
        if can_assemble(ctx, action_id).await {
            queued.push(action_id);
        }
    }
    // make sure there is one enqueued action
    assert_eq!(
        1,            //expected
        queued.len()  // actual
    );
    let func_id = Func::find_id_by_name(ctx, "test:createActionStarfield")
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("unable to get func")
        .expect("func is some");

    // detach the action
    for action_prototype_id in ActionPrototype::list_for_func_id(ctx, func_id)
        .await
        .expect("unable to list prototypes for func")
    {
        ActionPrototype::remove(ctx, action_prototype_id)
            .await
            .expect("unable to remove action prototype");
    }

    // check the func count for the schema variant is accurate
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    assert_eq!(
        initial_total_funcs - 1, // expected
        funcs.len()              // actual
    );

    // check that the action has been removed from the queue
    let enqueued_actions = Action::list_topologically(ctx)
        .await
        .expect("can list actions");
    let mut queued = Vec::new();

    for action_id in enqueued_actions.into_iter() {
        if can_assemble(ctx, action_id).await {
            queued.push(action_id);
        }
    }
    // make sure there aren't any enqueued actions
    assert_eq!(
        0,            //expected
        queued.len()  // actual
    );

    // reattach the create action, and enqueue it. All should work again
    let action_association = FuncAssociations::Action {
        kind: ActionKind::Create,
        schema_variant_ids: vec![schema_variant_id],
    };
    FuncAuthoringClient::save_func(
        ctx,
        func_id,
        Some(schema.name),
        func.name,
        func.description,
        func.code_base64,
        Some(action_association),
    )
    .await
    .expect("unable to save func");

    // ensure it got reattached
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    let total_funcs = funcs.len();

    assert_eq!(
        initial_total_funcs, // expected
        total_funcs,         // actual
    );

    // manually enqueue the action
    let mut action = None;
    for prototype in ActionPrototype::for_variant(ctx, schema_variant_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if prototype.kind == ActionKind::Create {
            action = Some(
                Action::new(ctx, prototype.id, Some(new_component.id()))
                    .await
                    .expect("unable to upsert action"),
            );
            break;
        }
    }
    // ensure we enqueued the action
    assert!(action.is_some());

    let enqueued_actions = Action::list_topologically(ctx)
        .await
        .expect("can list actions");
    let mut queued = Vec::new();

    for action_id in enqueued_actions.into_iter() {
        if can_assemble(ctx, action_id).await {
            queued.push(action_id);
        }
    }
    assert_eq!(
        1,            //expected
        queued.len()  // actual
    );

    // finally, delete the action
    let func_id = Func::find_id_by_name(ctx, "test:createActionStarfield")
        .await
        .expect("unable to find the func")
        .expect("no func found");
    // sdf calls this first if there are associations, so call it here too
    FuncAuthoringClient::detach_func_from_everywhere(ctx, func_id)
        .await
        .expect("could not detach func");
    Func::delete_by_id(ctx, func_id)
        .await
        .expect("unable to delete the func");

    // check the func count for the schema variant
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    assert_eq!(
        total_funcs - 1, // expected
        funcs.len()      // actual
    );

    // check that the action has been removed from the queue
    let enqueued_actions = Action::list_topologically(ctx)
        .await
        .expect("can list actions");
    let mut queued = Vec::new();

    for action_id in enqueued_actions.into_iter() {
        if can_assemble(ctx, action_id).await {
            queued.push(action_id);
        }
    }
    // make sure there aren't any enqueued actions
    assert_eq!(
        0,            //expected
        queued.len()  // actual
    );

    // let's create the func again
}

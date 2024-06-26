use dal::action::prototype::{ActionKind, ActionPrototype};
use dal::action::{Action, ActionId};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::binding::action::ActionBinding;
use dal::func::binding::FuncBindings;
use dal::func::summary::FuncSummary;
use dal::func::view::FuncView;
use dal::func::{FuncAssociations, FuncKind};
use dal::{DalContext, Func, Schema, SchemaVariant};
use dal_test::helpers::{create_component_for_schema_name, ChangeSetTestHelpers};
use dal_test::test;

#[test]
async fn attach_multiple_action_funcs(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "katy perry")
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

    // Attach one action func to the schema variant and commit.
    let func_id = Func::find_id_by_name(ctx, "test:createActionFallout")
        .await
        .expect("unable to find the func")
        .expect("no func found");

    ActionBinding::create_action_binding(ctx, func_id, ActionKind::Create, schema_variant_id)
        .await
        .expect("could not create action binding");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Attach a second action func to the same schema variant and commit.
    let func_id = Func::find_id_by_name(ctx, "test:deleteActionSwifty")
        .await
        .expect("unable to find the func")
        .expect("no func found");
    ActionBinding::create_action_binding(ctx, func_id, ActionKind::Destroy, schema_variant_id)
        .await
        .expect("could not create action binding");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Now, let's list all funcs and see the two that were attached.
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    assert_eq!(
        total_funcs + 2, // expected
        funcs.len()      // actual
    );
}

#[test]
async fn error_when_attaching_an_exisiting_type(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "fallout")
        .await
        .expect("unable to find by name")
        .expect("no schema found");
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");
    let func_id = Func::find_id_by_name(ctx, "test:createActionFallout")
        .await
        .expect("unable to find the func");
    assert!(func_id.is_some());

    let new_action_func_name = "anotherCreate";
    FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Action,
        Some(new_action_func_name.to_string()),
        None,
    )
    .await
    .expect("could not create func");

    let func_id = Func::find_id_by_name(ctx, new_action_func_name)
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("unable to get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("unable to assemble a func view");
    let (func_view_kind, mut schema_variant_ids) = func_view
        .associations
        .expect("empty associations")
        .get_action_internals()
        .expect("could not get internals");
    schema_variant_ids.push(schema_variant_id);
    assert!(FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        Some(FuncAssociations::Action {
            kind: func_view_kind,
            schema_variant_ids,
        }),
    )
    .await
    .is_err());
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
    let _func = Func::get_by_id(ctx, func_id)
        .await
        .expect("unable to get func")
        .expect("func is some");

    // detach the action
    for action_prototype_id in ActionPrototype::list_for_func_id(ctx, func_id)
        .await
        .expect("unable to list prototypes for func")
    {
        ActionBinding::delete_action_binding(ctx, action_prototype_id)
            .await
            .expect("could not delete action binding");
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
    ActionBinding::create_action_binding(ctx, func_id, ActionKind::Create, schema_variant_id)
        .await
        .expect("could not create action binding");

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
    FuncBindings::delete_all_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not delete bindings");

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
}

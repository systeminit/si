use dal::{
    DalContext,
    Func,
    SchemaVariant,
    action::{
        Action,
        ActionId,
        dependency_graph::ActionDependencyGraph,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
    func::{
        authoring::FuncAuthoringClient,
        binding::{
            FuncBinding,
            action::ActionBinding,
        },
    },
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_unlocked_schema_name_on_default_view,
    },
    test,
};
use si_db::FuncRunDb;

#[test]
async fn attach_multiple_action_funcs(ctx: &mut DalContext) {
    let schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, "katy perry")
        .await
        .expect("unable to get default schema variant");

    // Cache the total number of funcs before continuing.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");

    let total_funcs = funcs.len();

    // create unlocked copy of schema variant
    let schema_variant_id =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
            .await
            .expect("can create unlocked copy")
            .id();

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
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    assert_eq!(
        total_funcs + 2, // expected
        funcs.len()      // actual
    );
}

#[test]
async fn error_when_attaching_an_exisiting_type(ctx: &mut DalContext) {
    let schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, "fallout")
        .await
        .expect("unable to get default schema variant");
    let func_id = Func::find_id_by_name(ctx, "test:createActionFallout")
        .await
        .expect("unable to find the func");
    assert!(func_id.is_some());

    let new_action_func_name = "anotherCreate";

    assert!(
        FuncAuthoringClient::create_new_action_func(
            ctx,
            Some(new_action_func_name.to_owned()),
            ActionKind::Create,
            schema_variant_id,
        )
        .await
        .is_err()
    );
}

#[test]
async fn detach_attach_then_delete_action_func_while_enqueued(ctx: &mut DalContext) {
    pub async fn can_assemble(ctx: &DalContext, action_id: ActionId) -> bool {
        let action_graph = ActionDependencyGraph::for_workspace(ctx)
            .await
            .expect("could get action graph");
        let action = Action::get_by_id(ctx, action_id)
            .await
            .expect("unable to get action");
        let prototype_id = Action::prototype_id(ctx, action_id)
            .await
            .expect("unable to get prototype id");
        let _prototype = ActionPrototype::get_by_id(ctx, prototype_id)
            .await
            .expect("unable to get prototype");
        let _func_run_id = FuncRunDb::get_last_run_for_action_id_opt(
            ctx,
            ctx.events_tenancy().workspace_pk,
            action.id(),
        )
        .await
        .expect("unable to get func run id")
        .map(|f| f.id());
        let _component_id = Action::component_id(ctx, action_id)
            .await
            .expect("unable to get component id");
        let _my_dependencies = &action_graph.get_all_dependencies(action_id);
        let _dependent_on = &action_graph.direct_dependencies_of(action_id);
        let _hold_status_influenced_by =
            Action::get_hold_status_influenced_by(ctx, &action_graph, action_id)
                .await
                .expect("could get hold status");
        true
    }
    let old_schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, "starfield")
        .await
        .expect("unable to get default schema variant");

    // Cache the total number of funcs before continuing.
    let funcs = SchemaVariant::all_funcs(ctx, old_schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    let initial_total_funcs = funcs.len();
    // create unlocked copy
    let schema_variant_id =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, old_schema_variant_id)
            .await
            .expect("can create unlocked copy")
            .id();

    // create a component
    let new_component =
        create_component_for_unlocked_schema_name_on_default_view(ctx, "starfield", "component")
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
        .expect("unable to get func");

    let bindings = FuncBinding::for_func_id(ctx, func_id)
        .await
        .expect("found func bindings");
    // should be two bindings
    assert_eq!(2, bindings.len(),);

    // detach the action
    for action_binding in bindings {
        let FuncBinding::Action(action_binding) = action_binding else {
            panic!("wrong binding kind for Func")
        };
        let result =
            ActionBinding::delete_action_binding(ctx, action_binding.action_prototype_id).await;

        if action_binding.schema_variant_id == old_schema_variant_id {
            assert!(result.is_err());
        } else {
            result.expect("unable to delete prototype");
        }
    }

    // check the func count for the schema variant is accurate
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
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
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
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
}

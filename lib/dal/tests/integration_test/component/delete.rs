use std::time::Duration;

use dal::{
    Component,
    DalContext,
    Func,
    action::{
        Action,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
    component::{
        delete::{
            ComponentDeletionStatus,
            delete_components,
        },
        resource::ResourceData,
    },
    func::{
        authoring::FuncAuthoringClient,
        intrinsics::IntrinsicFunc,
    },
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    Result,
    expected::ExpectSchemaVariant,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        component,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use veritech_client::ResourceStatus;

#[test(enable_veritech)]
async fn delete(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
            .await
            .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert!(
        component
            .delete(ctx)
            .await
            .expect("unable to delete component")
            .is_none()
    );
}

#[test(enable_veritech)]
async fn delete_enqueues_destroy_action(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "dummy-secret", "component")
            .await
            .expect("could not create component");
    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );
    component
        .set_resource(ctx, resource_data)
        .await
        .expect("Unable to set resource");
    let schema_variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("Unable to get schema variant id");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    ActionPrototype::new(
        ctx,
        ActionKind::Destroy,
        "Destroy action".to_string(),
        None,
        schema_variant_id,
        Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
            .await
            .expect("Unable to find identity func"),
    )
    .await
    .expect("Unable to create destroy action");

    assert!(
        Action::all_ids(ctx)
            .await
            .expect("Unable to list enqueued actions")
            .is_empty()
    );

    component
        .delete(ctx)
        .await
        .expect("Unable to mark for deletion");

    let action_ids = Action::all_ids(ctx)
        .await
        .expect("Unable to list enqueued actions");
    assert_eq!(1, action_ids.len());
}

#[test(enable_veritech)]
async fn delete_on_already_to_delete_does_not_enqueue_destroy_action(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "dummy-secret", "component")
            .await
            .expect("could not create component");
    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );
    component
        .set_resource(ctx, resource_data)
        .await
        .expect("Unable to set resource");
    let schema_variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("Unable to get schema variant id");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    ActionPrototype::new(
        ctx,
        ActionKind::Destroy,
        "Destroy action".to_string(),
        None,
        schema_variant_id,
        Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
            .await
            .expect("Unable to find identity func"),
    )
    .await
    .expect("Unable to create destroy action");

    assert!(
        Action::all_ids(ctx)
            .await
            .expect("Unable to list enqueued actions")
            .is_empty()
    );

    let component = component
        .set_to_delete(ctx, true)
        .await
        .expect("Unable to set to_delete");

    let action_ids = Action::all_ids(ctx)
        .await
        .expect("Unable to list enqueued actions");
    assert_eq!(1, action_ids.len());
    for action_id in action_ids {
        Action::remove_by_id(ctx, action_id)
            .await
            .expect("Unable to remove action");
    }

    assert!(
        Action::all_ids(ctx)
            .await
            .expect("Unable to list enqueued actions")
            .is_empty()
    );

    component
        .delete(ctx)
        .await
        .expect("Unable to mark for deletion");

    assert!(
        Action::all_ids(ctx)
            .await
            .expect("Unable to list enqueued actions")
            .is_empty()
    );
}

// dependent_values_update::marked_for_deletion_to_normal_is_blocked tests delete downstream values

#[test(enable_veritech)]
async fn delete_multiple_components(ctx: &mut DalContext) -> Result<()> {
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    let component_still_on_head = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "component still on head",
    )
    .await?;

    // Remove any pending actions before applying to base to prevent
    // component_still_on_head from getting a resource via Create action
    let action_ids = Action::all_ids(ctx).await?;
    for action_id in action_ids {
        Action::remove_by_id(ctx, action_id).await?;
    }

    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    let component_with_resource_to_delete =
        create_component_for_default_schema_name_in_default_view(
            ctx,
            "small odd lego",
            "component with resource to delete",
        )
        .await?;

    let component_with_resource_to_erase =
        create_component_for_default_schema_name_in_default_view(
            ctx,
            "small odd lego",
            "component with resource to erase",
        )
        .await?;

    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );

    component_with_resource_to_delete
        .set_resource(ctx, resource_data.clone())
        .await?;
    component_with_resource_to_erase
        .set_resource(ctx, resource_data.clone())
        .await?;

    let component_to_delete = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "component to delete",
    )
    .await?;

    let expected_deletion_statuses = &[
        (component_to_delete.id(), ComponentDeletionStatus::Deleted),
        (
            component_with_resource_to_delete.id(),
            ComponentDeletionStatus::MarkedForDeletion,
        ),
        (
            component_still_on_head.id(),
            ComponentDeletionStatus::StillExistsOnHead,
        ),
        (
            component_with_resource_to_erase.id(),
            ComponentDeletionStatus::Deleted,
        ),
    ];

    let mut deletion_statuses = delete_components(
        ctx,
        &[
            component_to_delete.id(),
            component_with_resource_to_delete.id(),
            component_still_on_head.id(),
        ],
        false,
    )
    .await?;

    deletion_statuses
        .extend(delete_components(ctx, &[component_with_resource_to_erase.id()], true).await?);

    for (component_id, status) in expected_deletion_statuses {
        assert_eq!(Some(status), deletion_statuses.get(component_id));
    }

    assert!(
        Component::try_get_by_id(ctx, component_to_delete.id())
            .await?
            .is_none(),
        "deleted component should be gone"
    );

    assert!(
        Component::try_get_by_id(ctx, component_still_on_head.id())
            .await?
            .is_none(),
        "deleted component that is still on head should be gone in this change set"
    );

    assert!(
        Component::exists_on_head_by_ids(ctx, &[component_still_on_head.id()])
            .await?
            .contains(&component_still_on_head.id()),
        "component should still exist on head"
    );

    assert!(
        Component::try_get_by_id(ctx, component_with_resource_to_erase.id())
            .await?
            .is_none(),
        "erased component should be gone"
    );

    let component_with_resource_to_delete =
        Component::get_by_id(ctx, component_with_resource_to_delete.id()).await?;
    assert!(
        component_with_resource_to_delete.to_delete(),
        "component with resource should be marked as to delete"
    );

    Ok(())
}

#[test(enable_veritech)]
async fn delete_multiple_components_with_subscriptions(ctx: &mut DalContext) -> Result<()> {
    // Create a component B that feeds 2 other components A via subscription
    // Run Create actions for 2 components A
    // Delete all 3 of them (which should mark them all as to_delete)
    // Check that the first component isn't allowed to be removed since the downstream components need them
    // Check that all 3 components are deleted after the delete actions run!

    let component_a_code_definition = r#"
        function main() {
            const prop = new PropBuilder()
                .setName("prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .build();
            const resourceProp = new PropBuilder()
                .setName("prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .build();

            return new AssetBuilder()
                .addProp(prop)
                .addResourceProp(resourceProp)
                .build();
        }
    "#;

    let a_variant = ExpectSchemaVariant(
        VariantAuthoringClient::create_schema_and_variant_from_code(
            ctx,
            "A",
            None,
            None,
            "Category",
            "#0077cc",
            component_a_code_definition,
        )
        .await?
        .id,
    );

    // Create Action Func for A
    let func_name = "Create A".to_string();
    let func = FuncAuthoringClient::create_new_action_func(
        ctx,
        Some(func_name.clone()),
        ActionKind::Create,
        a_variant.id(),
    )
    .await?;

    let create_func_code = r#"
        async function main(component: Input): Promise<Output> {
        const prop = component.properties.domain?.prop;
    return {
        status: "ok",
        payload: {
            prop: prop
        },
    }
}
    "#;
    FuncAuthoringClient::save_code(ctx, func.id, create_func_code).await?;
    // Create Action Func for A
    let func_name = "Destroy A".to_string();
    let func = FuncAuthoringClient::create_new_action_func(
        ctx,
        Some(func_name.clone()),
        ActionKind::Destroy,
        a_variant.id(),
    )
    .await?;

    let delete_func_code = r#"
        async function main(component: Input): Promise<Output> {
    return {
        status: "ok",
        payload: null,
    }
}
    "#;
    FuncAuthoringClient::save_code(ctx, func.id, delete_func_code).await?;

    // Create B Variant
    let component_b_code_definition = r#"
        function main() {
            const prop = new PropBuilder()
                .setName("prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .build();
            return new AssetBuilder()
                .addProp(prop)
                .build();
        }
    "#;

    let _b_variant = ExpectSchemaVariant(
        VariantAuthoringClient::create_schema_and_variant_from_code(
            ctx,
            "B",
            None,
            None,
            "Category",
            "#0077cc",
            component_b_code_definition,
        )
        .await?
        .id,
    );
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // create 1 component B and 2 component As
    let a_1 = component::create(ctx, "A", "A1").await?;
    let a_2 = component::create(ctx, "A", "A2").await?;
    let b = component::create(ctx, "B", "B").await?;

    // both A's subscribe to B
    value::subscribe(ctx, ("A1", "/domain/prop"), ("B", "/domain/prop")).await?;
    value::subscribe(ctx, ("A2", "/domain/prop"), ("B", "/domain/prop")).await?;

    // update value for B
    value::set(ctx, ("B", "/domain/prop"), "hello world").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert!(value::has_value(ctx, ("A1", "/domain/prop")).await?);
    assert!(value::has_value(ctx, ("A2", "/domain/prop")).await?);

    let actions = Action::list_topologically(ctx).await?;
    assert!(actions.len() == 2);
    // Apply changeset so it runs the creation action
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    // wait for actions to run
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;
    // fork head
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    let actions = Action::list_topologically(ctx).await?;
    assert!(actions.is_empty());
    assert!(value::has_value(ctx, ("A1", "/resource/payload")).await?);
    assert!(value::has_value(ctx, ("A2", "/resource/payload")).await?);

    // now delete all 3 components
    let a_1_comp = Component::get_by_id(ctx, a_1).await?.delete(ctx).await?;
    let a_2_comp = Component::get_by_id(ctx, a_2).await?.delete(ctx).await?;
    let b_comp = Component::get_by_id(ctx, b).await?.delete(ctx).await?;

    assert!(a_1_comp.is_some());
    assert!(a_2_comp.is_some());
    assert!(b_comp.is_some());

    // now should have 2 delete actions enqueued
    let actions = Action::list_topologically(ctx).await?;
    assert!(actions.len() == 2);

    // Apply changeset so it runs the creation action
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    // wait for actions to run
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;
    // loop until the other components are removed
    let total_count = 50;
    let mut count = 0;

    while count < total_count {
        ctx.update_snapshot_to_visibility()
            .await
            .expect("could not update snapshot");
        let components = Component::list(ctx)
            .await
            .expect("could not list components");
        if components.is_empty() {
            break;
        }
        count += 1;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    // All components are gone!
    assert!(Component::list(ctx).await?.is_empty());

    Ok(())
}

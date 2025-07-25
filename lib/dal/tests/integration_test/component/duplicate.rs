use dal::{
    Component,
    DalContext,
    diagram::view::View,
};
use dal_test::{
    Result,
    expected::ExpectComponent,
    helpers::ChangeSetTestHelpers,
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

use crate::integration_test::component::connectable_test::{
    Subscribable,
    SubscribableTest,
};

#[test]
async fn duplicate_component_with_value(ctx: &mut DalContext) -> Result<()> {
    let component = ExpectComponent::create_named(ctx, "pirate", "Long John Silver").await;
    let parrots = component
        .prop(ctx, ["root", "domain", "parrot_names"])
        .await;

    // set value on pet shop component
    parrots.push(ctx, "Captain Flint").await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert!(parrots.has_value(ctx).await);

    let default_view_id = View::get_id_for_default(ctx).await?;

    // Duplicate the pirate component
    let duplicated_ids = Component::duplicate(ctx, default_view_id, vec![component.id()]).await?;
    assert_eq!(duplicated_ids.len(), 1);

    let component_copy = ExpectComponent(duplicated_ids[0]);
    let parrots_copy = component_copy.prop(ctx, parrots).await;

    assert_ne!(component.id(), component_copy.id());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Validate that component_copy has the new value
    assert!(parrots_copy.has_value(ctx).await);
    assert_eq!(json!(["Captain Flint"]), parrots_copy.get(ctx).await);

    assert!(parrots.has_value(ctx).await);

    Ok(())
}

#[test]
async fn duplicate_component_with_dependent_value(ctx: &mut DalContext) -> Result<()> {
    let source = ExpectComponent::create_named(ctx, "pet_shop", "Petopia").await;
    let downstream = ExpectComponent::create_named(ctx, "pirate", "Long John Silver").await;
    let source_parrots = source.prop(ctx, ["root", "domain", "parrot_names"]).await;
    let downstream_parrots = downstream
        .prop(ctx, ["root", "domain", "parrot_names"])
        .await;

    // set value on source component
    source_parrots.push(ctx, "Captain Flint").await;
    source
        .connect(ctx, "parrot_names", downstream, "parrot_names")
        .await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that downstream has the parrots value, and that it is not explicitly set
    assert!(downstream_parrots.has_value(ctx).await);
    assert_eq!(
        Some(json!(["Captain Flint"])),
        downstream_parrots.view(ctx).await
    );

    let default_view_id = View::get_id_for_default(ctx).await?;

    // Duplicate the downstream component
    let duplicated_ids = Component::duplicate(ctx, default_view_id, vec![downstream.id()]).await?;
    assert_eq!(duplicated_ids.len(), 1);

    let downstream_copy = ExpectComponent(duplicated_ids[0]);
    let downstream_copy_parrots = downstream_copy.prop(ctx, downstream_parrots).await;

    assert_ne!(downstream.id(), downstream_copy.id());

    // Check that the copy does *not* have the parrots value, because it is not explicitly set
    // (because it has no link - duplicate ignores connections)
    assert!(!downstream_copy_parrots.has_value(ctx).await);
    assert_eq!(None, downstream_copy_parrots.view(ctx).await);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that the copy does *not* have the parrots value, because it is not explicitly set
    // (because it has no link - duplicate ignores connections)
    assert!(!downstream_copy_parrots.has_value(ctx).await);
    assert_eq!(None, downstream_copy_parrots.view(ctx).await);

    assert!(downstream_parrots.has_value(ctx).await);
    assert_eq!(
        Some(json!(["Captain Flint"])),
        downstream_parrots.view(ctx).await
    );

    assert_eq!(
        Some(json!({
            "domain": {
                // Propagated from /si/name, which means the attribute prototype has been copied
                // from the copied component (since we manually set all values, which removes the
                // default attribute prototype for the slot
                "name": "Long John Silver - Copy",

                // The connection is not copied (duplicate ignores connections)
                // "parrot_names": [
                //     "Captain Flint",
                // ],
            },
            "resource_value": {},
            "resource": {},
            "si": {
                "color": "#ff00ff",
                "name": "Long John Silver - Copy",
                "type": "component",
            },
        })),
        downstream_copy.view(ctx).await,
    );

    Ok(())
}

#[test]
async fn duplicate_components_with_subscriptions(ctx: &mut DalContext) -> Result<()> {
    let test = SubscribableTest::setup(ctx).await?;

    // input1
    let input1 = test.create_subscribable(ctx, "input1", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input1"
        }),
        input1.domain(ctx).await?
    );

    // input2
    let input2 = test.create_subscribable(ctx, "input2", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input2"
        }),
        input2.domain(ctx).await?
    );

    // original1 with subscriptions to input1 and input2
    let original1 = test
        .create_subscribable(ctx, "original1", Some(input1), [input1, input2])
        .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        json!({
            "Value": "original1",
            "One": "input1",
            "Many": ["input1", "input2"],
        }),
        original1.domain(ctx).await?
    );

    // original2 with subscriptions to original1 and inputs
    let original2 = test
        .create_subscribable(
            ctx,
            "original2",
            Some(original1),
            [input1, input2, original1],
        )
        .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        json!({
            "Value": "original2",
            "One": "original1",
            "Many": ["input1", "input2", "original1"],
        }),
        original2.domain(ctx).await?
    );

    // Duplicate original1 and original2
    let duplicated_ids = Component::duplicate(
        ctx,
        View::get_id_for_default(ctx).await?,
        vec![original1.id, original2.id],
    )
    .await?;
    assert_eq!(duplicated_ids.len(), 2);

    let duplicated1 = Subscribable::new(test, duplicated_ids[0]);
    let duplicated2 = Subscribable::new(test, duplicated_ids[1]);

    // Set the duplicated components' values to make sure those are flowing as expected
    duplicated1.set_value(ctx, "duplicated1").await?;
    duplicated2.set_value(ctx, "duplicated2").await?;

    // Set the external components' values to new values to ensure they flow through
    // any remaining subscriptions
    input1.set_value(ctx, "input1-new").await?;
    input2.set_value(ctx, "input2-new").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure original1 and original2 didn't change their behavior
    assert_eq!(
        json!({
            "Value": "original1",
            "One": "input1-new",
            "Many": ["input1-new", "input2-new"],
        }),
        original1.domain(ctx).await?
    );
    assert_eq!(
        json!({
            "Value": "original2",
            "One": "original1",
            "Many": ["input1-new", "input2-new", "original1"],
        }),
        original2.domain(ctx).await?
    );

    // The duplicated components should have all subscriptions preserved:
    // - External subscriptions to non-duplicated components (input1, input2) are preserved
    // - Internal subscriptions between duplicated components are also preserved
    assert_eq!(
        json!({
            "Value": "duplicated1",
            "One": "input1-new",
            "Many": ["input1-new", "input2-new"],
        }),
        duplicated1.domain(ctx).await?
    );

    // duplicated2 should get values from both external sources AND duplicated1
    // because duplicate preserves internal subscriptions between duplicated components
    assert_eq!(
        json!({
            "Value": "duplicated2",
            "One": "duplicated1", // Should have "One" field from duplicated1
            "Many": ["duplicated1", "input1-new", "input2-new"], // Should include duplicated1
        }),
        duplicated2.domain(ctx).await?
    );

    Ok(())
}

#[test]
async fn duplicate_manager_and_managed(ctx: &mut DalContext) -> Result<()> {
    let test = SubscribableTest::setup(ctx).await?;

    // manager and original
    let manager = test.create_manager(ctx, "manager").await?;
    let original = test.create_subscribable(ctx, "original", None, []).await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Duplicate manager and original together
    let duplicated_ids = Component::duplicate(
        ctx,
        View::get_id_for_default(ctx).await?,
        vec![manager.id, original.id],
    )
    .await?;
    assert_eq!(duplicated_ids.len(), 2);

    let duplicated_manager = Subscribable::new(test, duplicated_ids[0]);
    let duplicated_original = Subscribable::new(test, duplicated_ids[1]);

    // Set the duplicated component's value so we can tell the difference
    duplicated_original.set_value(ctx, "duplicated").await?;
    duplicated_manager
        .set_value(ctx, "duplicated manager")
        .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure the originals are unaltered
    assert_eq!(json!(["original"]), manager.run_management_func(ctx).await?);

    // The duplicated components should have management relationships preserved
    assert_eq!(
        json!(["duplicated"]),
        duplicated_manager.run_management_func(ctx).await?
    );

    Ok(())
}

#[test]
async fn duplicate_manager_only(ctx: &mut DalContext) -> Result<()> {
    let test = SubscribableTest::setup(ctx).await?;

    // manager and original
    let manager = test.create_manager(ctx, "manager").await?;
    let original = test.create_subscribable(ctx, "original", None, []).await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Duplicate only the manager
    let duplicated_ids =
        Component::duplicate(ctx, View::get_id_for_default(ctx).await?, vec![manager.id]).await?;
    assert_eq!(duplicated_ids.len(), 1);

    let duplicated_manager = Subscribable::new(test, duplicated_ids[0]);

    // Set the duplicated component's value so we can tell the difference
    duplicated_manager
        .set_value(ctx, "duplicated manager")
        .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure the originals are unaltered
    assert_eq!(json!(["original"]), manager.run_management_func(ctx).await?);

    // The duplicated manager should have no managed components
    // (since we didn't duplicate the managed component)
    assert_eq!(
        json!([]),
        duplicated_manager.run_management_func(ctx).await?
    );

    Ok(())
}

#[test]
async fn duplicate_managed_only(ctx: &mut DalContext) -> Result<()> {
    let test = SubscribableTest::setup(ctx).await?;

    // manager and original
    let manager = test.create_manager(ctx, "manager").await?;
    let original = test.create_subscribable(ctx, "original", None, []).await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Duplicate only the managed component
    let duplicated_ids =
        Component::duplicate(ctx, View::get_id_for_default(ctx).await?, vec![original.id]).await?;
    assert_eq!(duplicated_ids.len(), 1);

    let duplicated_original = Subscribable::new(test, duplicated_ids[0]);

    // Set the duplicated component's value so we can tell the difference
    duplicated_original.set_value(ctx, "duplicated").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // The original manager should only be managing the original component, not the duplicate
    let managed_components = manager.run_management_func(ctx).await?;
    assert_eq!(1, managed_components.as_array().unwrap().len());
    // The array should contain only "original"
    assert!(
        managed_components
            .as_array()
            .unwrap()
            .contains(&json!("original"))
    );

    Ok(())
}

#[test]
async fn duplicate_preserves_external_and_recreates_internal_subscriptions(
    ctx: &mut DalContext,
) -> Result<()> {
    let test = SubscribableTest::setup(ctx).await?;

    // Create external components that won't be duplicated
    let external1 = test.create_subscribable(ctx, "external1", None, []).await?;
    let external2 = test.create_subscribable(ctx, "external2", None, []).await?;

    // Create components that will be duplicated with both external and internal subscriptions
    let original_a = test
        .create_subscribable(ctx, "original_a", Some(external1), [external1, external2])
        .await?;
    let original_b = test
        .create_subscribable(
            ctx,
            "original_b",
            Some(original_a),
            [external1, external2, original_a],
        )
        .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify initial state
    assert_eq!(
        json!({
            "Value": "original_a",
            "One": "external1",           // External subscription
            "Many": ["external1", "external2"], // External subscriptions
        }),
        original_a.domain(ctx).await?
    );

    assert_eq!(
        json!({
            "Value": "original_b",
            "One": "original_a",         // Internal subscription (to original_a)
            "Many": ["external1", "external2", "original_a"], // Mixed: external + internal
        }),
        original_b.domain(ctx).await?
    );

    // Duplicate both components
    let duplicated_ids = Component::duplicate(
        ctx,
        View::get_id_for_default(ctx).await?,
        vec![original_a.id, original_b.id],
    )
    .await?;
    assert_eq!(duplicated_ids.len(), 2);

    let duplicated_a = Subscribable::new(test, duplicated_ids[0]);
    let duplicated_b = Subscribable::new(test, duplicated_ids[1]);

    // Set unique values for duplicated components
    duplicated_a.set_value(ctx, "duplicated_a").await?;
    duplicated_b.set_value(ctx, "duplicated_b").await?;

    // Change external component values to verify external subscriptions work
    external1.set_value(ctx, "external1_updated").await?;
    external2.set_value(ctx, "external2_updated").await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify originals still work correctly
    assert_eq!(
        json!({
            "Value": "original_a",
            "One": "external1_updated",
            "Many": ["external1_updated", "external2_updated"],
        }),
        original_a.domain(ctx).await?
    );

    assert_eq!(
        json!({
            "Value": "original_b",
            "One": "original_a",        // Still points to original_a
            "Many": ["external1_updated", "external2_updated", "original_a"],
        }),
        original_b.domain(ctx).await?
    );

    // Verify duplicated components have correct subscription behavior:
    // 1. External subscriptions are maintained (to external1, external2)
    // 2. Internal subscriptions are recreated (duplicated_b now points to duplicated_a)
    assert_eq!(
        json!({
            "Value": "duplicated_a",
            "One": "external1_updated",    // External subscription maintained
            "Many": ["external1_updated", "external2_updated"], // External subscriptions maintained
        }),
        duplicated_a.domain(ctx).await?
    );

    assert_eq!(
        json!({
            "Value": "duplicated_b",
            "One": "duplicated_a",         // Internal subscription recreated (now points to duplicated_a, not original_a)
            "Many": ["duplicated_a", "external1_updated", "external2_updated"], // Mixed: recreated internal + maintained external
        }),
        duplicated_b.domain(ctx).await?
    );

    // Verify that changing duplicated_a affects duplicated_b but not original_b
    duplicated_a.set_value(ctx, "duplicated_a_changed").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // original_b should be unaffected by changes to duplicated_a
    assert_eq!(
        json!({
            "Value": "original_b",
            "One": "original_a",        // Still points to original_a (unchanged)
            "Many": ["external1_updated", "external2_updated", "original_a"],
        }),
        original_b.domain(ctx).await?
    );

    // duplicated_b should reflect the change to duplicated_a
    assert_eq!(
        json!({
            "Value": "duplicated_b",
            "One": "duplicated_a_changed", // Reflects change to duplicated_a
            "Many": ["duplicated_a_changed", "external1_updated", "external2_updated"],
        }),
        duplicated_b.domain(ctx).await?
    );

    Ok(())
}

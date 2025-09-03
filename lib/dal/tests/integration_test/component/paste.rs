use dal::{
    Component,
    DalContext,
    diagram::view::View,
};
use dal_test::{
    Result,
    expected::ExpectComponent,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

use crate::integration_test::component::connectable_test::{
    Connectable,
    ConnectableTest,
    GEOMETRY1,
    GEOMETRY2,
};

#[test]
async fn paste_component_with_value(ctx: &mut DalContext) -> Result<()> {
    let component = ExpectComponent::create_named(ctx, "pirate", "Long John Silver").await;
    let parrots = component
        .prop(ctx, ["root", "domain", "parrot_names"])
        .await;

    // set value on pet shop component
    parrots.push(ctx, "Captain Flint").await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert!(parrots.has_value(ctx).await);

    let default_view_id = View::get_id_for_default(ctx).await?;

    // Copy/paste the pirate component
    let component_copy = ExpectComponent(
        component
            .component(ctx)
            .await
            .duplicate_without_connections(
                ctx,
                default_view_id,
                component.geometry_for_default(ctx).await,
                None,
            )
            .await?
            .id(),
    );
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
async fn paste_components_with_subscriptions(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // input1
    let input1 = test.create_connectable(ctx, "input1").await?;
    assert_eq!(
        json!({
            "Value": "input1"
        }),
        input1.domain(ctx).await?
    );

    // input2
    let input2 = test.create_connectable(ctx, "input2").await?;
    assert_eq!(
        json!({
            "Value": "input2"
        }),
        input2.domain(ctx).await?
    );

    // input1 -> original1
    let original1 = test.create_connectable(ctx, "original1").await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/One"),
        ("input1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/Many/-"),
        ("input1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/Many/-"),
        ("input2", "/domain/Value"),
    )
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

    // original1 -> original2
    let original2 = test.create_connectable(ctx, "original2").await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/One"),
        ("original1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/Many/-"),
        ("input1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/Many/-"),
        ("input2", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/Many/-"),
        ("original1", "/domain/Value"),
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

    // original1 -> output
    let output = test.create_connectable(ctx, "output").await?;
    value::subscribe(
        ctx,
        ("output", "/domain/One"),
        ("original2", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many/-"),
        ("input1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many/-"),
        ("input2", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many/-"),
        ("original1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many/-"),
        ("original2", "/domain/Value"),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        json!({
            "Value": "output",
            "One": "original2",
            "Many": ["input1", "input2", "original1", "original2"],
        }),
        output.domain(ctx).await?
    );

    // Copy/paste original2/1 -> pasted2/1
    let (pasted1, pasted2) = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            vec![(original1.id, GEOMETRY2), (original2.id, GEOMETRY1)],
        )
        .await?;
        assert_eq!(pasted.len(), 2);
        (
            Connectable::new(test, pasted[0]),
            Connectable::new(test, pasted[1]),
        )
    };

    // Set the pasted components' values to make sure those are flowing as expected
    pasted1.set_value(ctx, "pasted1").await?;
    pasted2.set_value(ctx, "pasted2").await?;

    // Set the external components' values to new values to ensure they flow through the pasted
    // connections
    input1.set_value(ctx, "input1-new").await?;
    input2.set_value(ctx, "input2-new").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure original1 and original2 didn't change
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

    // Make sure the pasted components got the connected values we expect
    assert_eq!(
        json!({
            "Value": "pasted1",
            "One": "input1-new",
            "Many": ["input1-new", "input2-new"],
        }),
        pasted1.domain(ctx).await?
    );
    assert_eq!(
        json!({
            "Value": "pasted2",
            "One": "pasted1",
            "Many": ["input1-new", "input2-new", "pasted1"],
        }),
        pasted2.domain(ctx).await?
    );

    // Make sure the pasted components were *not* connected to the output
    assert_eq!(
        json!({
            "Value": "output",
            "One": "original2",
            // TODO incorrect
            "Many": ["input1-new", "input2-new", "original1", "original2"],
        }),
        output.domain(ctx).await?
    );

    Ok(())
}

#[test]
async fn paste_components_with_subscriptions_opposite_order(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // input1
    let input1 = test.create_connectable(ctx, "input1").await?;
    assert_eq!(
        json!({
            "Value": "input1"
        }),
        input1.domain(ctx).await?
    );

    // input2
    let input2 = test.create_connectable(ctx, "input2").await?;
    assert_eq!(
        json!({
            "Value": "input2"
        }),
        input2.domain(ctx).await?
    );

    // input1 -> original1
    let original1 = test.create_connectable(ctx, "original1").await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/One"),
        ("input1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/Many/-"),
        ("input1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/Many/-"),
        ("input2", "/domain/Value"),
    )
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

    // original1 -> original2
    let original2 = test.create_connectable(ctx, "original2").await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/One"),
        ("original1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/Many/-"),
        ("input1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/Many/-"),
        ("input2", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/Many/-"),
        ("original1", "/domain/Value"),
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

    // original1 -> output
    let output = test.create_connectable(ctx, "output").await?;
    value::subscribe(
        ctx,
        ("output", "/domain/One"),
        ("original2", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many/-"),
        ("input1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many/-"),
        ("input2", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many/-"),
        ("original1", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many/-"),
        ("original2", "/domain/Value"),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        json!({
            "Value": "output",
            "One": "original2",
            "Many": ["input1", "input2", "original1", "original2"],
        }),
        output.domain(ctx).await?
    );

    // Copy/paste original2/1 -> pasted2/1
    let (pasted2, pasted1) = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            vec![(original2.id, GEOMETRY2), (original1.id, GEOMETRY1)],
        )
        .await?;
        assert_eq!(pasted.len(), 2);
        (
            Connectable::new(test, pasted[0]),
            Connectable::new(test, pasted[1]),
        )
    };

    // Set the pasted components' values to make sure those are flowing as expected
    pasted1.set_value(ctx, "pasted1").await?;
    pasted2.set_value(ctx, "pasted2").await?;

    // Set the external components' values to new values to ensure they flow through the pasted
    // connections
    input1.set_value(ctx, "input1-new").await?;
    input2.set_value(ctx, "input2-new").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure original1 and original2 didn't change
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

    // Make sure the pasted components got the connected values we expect
    assert_eq!(
        json!({
            "Value": "pasted1",
            "One": "input1-new",
            "Many": ["input1-new", "input2-new"],
        }),
        pasted1.domain(ctx).await?
    );
    assert_eq!(
        json!({
            "Value": "pasted2",
            "One": "pasted1",
            "Many": ["input1-new", "input2-new", "pasted1"],
        }),
        pasted2.domain(ctx).await?
    );

    // Make sure the pasted components were *not* connected to the output
    assert_eq!(
        json!({
            "Value": "output",
            "One": "original2",
            // TODO incorrect
            "Many": ["input1-new", "input2-new", "original1", "original2"],
        }),
        output.domain(ctx).await?
    );

    Ok(())
}

#[test]
async fn paste_manager_and_managed(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // manager and original
    let manager = test.create_manager(ctx, "manager").await?;
    let original = test.create_connectable(ctx, "original").await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Copy/paste manager/original -> pasted_manager/pasted
    let (pasted_manager, pasted) = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            vec![(manager.id, GEOMETRY1), (original.id, GEOMETRY2)],
        )
        .await?;
        assert_eq!(pasted.len(), 2);
        (
            Connectable::new(test, pasted[0]),
            Connectable::new(test, pasted[1]),
        )
    };

    // Set the pasted component's value so we can tell the difference
    pasted.set_value(ctx, "pasted").await?;
    pasted_manager.set_value(ctx, "pasted manager").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure the originals are unaltered
    assert_eq!(json!(["original"]), manager.run_management_func(ctx).await?);

    // Make sure the pasted components are managed
    assert_eq!(
        json!(["pasted"]),
        pasted_manager.run_management_func(ctx).await?
    );

    Ok(())
}

#[test]
async fn paste_manager_and_managed_opposite_order(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // manager and original
    let manager = test.create_manager(ctx, "manager").await?;
    let original = test.create_connectable(ctx, "original").await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Copy/paste original/manager -> pasted/pasted_manager
    let (pasted, pasted_manager) = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            vec![(original.id, GEOMETRY1), (manager.id, GEOMETRY2)],
        )
        .await?;
        assert_eq!(pasted.len(), 2);
        (
            Connectable::new(test, pasted[0]),
            Connectable::new(test, pasted[1]),
        )
    };

    // Set the pasted component's value so we can tell the difference
    pasted.set_value(ctx, "pasted").await?;
    pasted_manager.set_value(ctx, "pasted manager").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure the originals are unaltered
    assert_eq!(json!(["original"]), manager.run_management_func(ctx).await?);

    // Make sure the pasted components are managed
    assert_eq!(
        json!(["pasted"]),
        pasted_manager.run_management_func(ctx).await?
    );

    Ok(())
}

#[test]
async fn paste_manager(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // manager and original
    let manager = test.create_manager(ctx, "manager").await?;
    let original = test.create_connectable(ctx, "original").await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Copy/paste manager -> pasted_manager
    let pasted_manager = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            vec![(manager.id, GEOMETRY1)],
        )
        .await?;
        assert_eq!(pasted.len(), 1);
        Connectable::new(test, pasted[0])
    };

    // Set the pasted component's value so we can tell the difference
    pasted_manager.set_value(ctx, "pasted manager").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure the originals are unaltered
    assert_eq!(json!(["original"]), manager.run_management_func(ctx).await?);

    // Make sure the pasted component has no managed components
    assert_eq!(json!([]), pasted_manager.run_management_func(ctx).await?);

    Ok(())
}

#[test]
async fn paste_managed(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // manager and original
    let manager = test.create_manager(ctx, "manager").await?;
    let original = test.create_connectable(ctx, "original").await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Copy/paste original -> pasted
    let pasted = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            vec![(original.id, GEOMETRY1)],
        )
        .await?;
        assert_eq!(pasted.len(), 1);
        Connectable::new(test, pasted[0])
    };

    // Set the pasted component's value so we can tell the difference
    pasted.set_value(ctx, "pasted").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure the original management function is unaltered
    assert_eq!(
        json!(["original", "pasted"]),
        manager.run_management_func(ctx).await?
    );

    Ok(())
}

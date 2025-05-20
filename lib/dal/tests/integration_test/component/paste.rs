use dal::{
    Component,
    DalContext,
    component::frame::Frame,
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
            .copy_without_connections(
                ctx,
                default_view_id,
                component.geometry_for_default(ctx).await,
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
async fn paste_component_with_dependent_value(ctx: &mut DalContext) -> Result<()> {
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

    // Copy/paste the downstream component
    let downstream_copy = ExpectComponent(
        downstream
            .component(ctx)
            .await
            .copy_without_connections(
                ctx,
                default_view_id,
                downstream.geometry_for_default(ctx).await,
            )
            .await?
            .id(),
    );
    let downstream_copy_parrots = downstream_copy.prop(ctx, downstream_parrots).await;

    assert_ne!(downstream.id(), downstream_copy.id());

    // Check that the copy does *not* have the parrots value, because it is not explicitly set
    // (because it has no link)
    assert!(!downstream_copy_parrots.has_value(ctx).await);
    assert_eq!(None, downstream_copy_parrots.view(ctx).await);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that the copy does *not* have the parrots value, because it is not explicitly set
    // (because it has no link)
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

                // The connection is not copied
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
async fn paste_components_with_connections(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // input1
    let input1 = test.create_connectable(ctx, "input1", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input1"
        }),
        input1.domain(ctx).await?
    );

    // input2
    let input2 = test.create_connectable(ctx, "input2", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input2"
        }),
        input2.domain(ctx).await?
    );

    // input1 -> original1
    let original1 = test
        .create_connectable(ctx, "original1", Some(input1), [input1, input2])
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
    let original2 = test
        .create_connectable(
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

    // original1 -> output
    let output = test
        .create_connectable(
            ctx,
            "output",
            Some(original2),
            [input1, input2, original1, original2],
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
            None,
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
async fn paste_components_with_connections_opposite_order(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // let out_one = OutputSocket::find_with_name_or_error(ctx, "One", connectable.id()).await?;
    // let one = InputSocket::find_with_name_or_error(ctx, "One", connectable.id()).await?;

    // input1
    let input1 = test.create_connectable(ctx, "input1", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input1"
        }),
        input1.domain(ctx).await?
    );

    // input2
    let input2 = test.create_connectable(ctx, "input2", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input2"
        }),
        input2.domain(ctx).await?
    );

    // input1 -> original1
    let original1 = test
        .create_connectable(ctx, "original1", Some(input1), [input1, input2])
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
    let original2 = test
        .create_connectable(
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

    // original1 -> output
    let output = test
        .create_connectable(
            ctx,
            "output",
            Some(original2),
            [input1, input2, original1, original2],
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
            None,
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
async fn paste_components_with_subscriptions(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // input1
    let input1 = test.create_connectable(ctx, "input1", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input1"
        }),
        input1.domain(ctx).await?
    );

    // input2
    let input2 = test.create_connectable(ctx, "input2", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input2"
        }),
        input2.domain(ctx).await?
    );

    // input1 -> original1
    let original1 = test.create_connectable(ctx, "original1", None, []).await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/One"),
        [("input1", "/domain/Value")],
    )
    .await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/Many"),
        [("input1", "/domain/Value"), ("input2", "/domain/Value")],
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
    let original2 = test.create_connectable(ctx, "original2", None, []).await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/One"),
        [("original1", "/domain/Value")],
    )
    .await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/Many"),
        [
            ("input1", "/domain/Value"),
            ("input2", "/domain/Value"),
            ("original1", "/domain/Value"),
        ],
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
    let output = test.create_connectable(ctx, "output", None, []).await?;
    value::subscribe(
        ctx,
        ("output", "/domain/One"),
        [("original2", "/domain/Value")],
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many"),
        [
            ("input1", "/domain/Value"),
            ("input2", "/domain/Value"),
            ("original1", "/domain/Value"),
            ("original2", "/domain/Value"),
        ],
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
            None,
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
    let input1 = test.create_connectable(ctx, "input1", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input1"
        }),
        input1.domain(ctx).await?
    );

    // input2
    let input2 = test.create_connectable(ctx, "input2", None, []).await?;
    assert_eq!(
        json!({
            "Value": "input2"
        }),
        input2.domain(ctx).await?
    );

    // input1 -> original1
    let original1 = test.create_connectable(ctx, "original1", None, []).await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/One"),
        [("input1", "/domain/Value")],
    )
    .await?;
    value::subscribe(
        ctx,
        ("original1", "/domain/Many"),
        [("input1", "/domain/Value"), ("input2", "/domain/Value")],
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
    let original2 = test.create_connectable(ctx, "original2", None, []).await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/One"),
        [("original1", "/domain/Value")],
    )
    .await?;
    value::subscribe(
        ctx,
        ("original2", "/domain/Many"),
        [
            ("input1", "/domain/Value"),
            ("input2", "/domain/Value"),
            ("original1", "/domain/Value"),
        ],
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
    let output = test.create_connectable(ctx, "output", None, []).await?;
    value::subscribe(
        ctx,
        ("output", "/domain/One"),
        [("original2", "/domain/Value")],
    )
    .await?;
    value::subscribe(
        ctx,
        ("output", "/domain/Many"),
        [
            ("input1", "/domain/Value"),
            ("input2", "/domain/Value"),
            ("original1", "/domain/Value"),
            ("original2", "/domain/Value"),
        ],
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
            None,
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
async fn paste_child_and_parent(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // parent and child
    let parent = test.create_parent(ctx, "parent").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let child = test.create_connectable(ctx, "child", None, []).await?;
    Frame::upsert_parent(ctx, child.id, parent.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        json!({
            "Value": "child",
            "Inferred": "parent",
        }),
        child.domain(ctx).await?
    );

    // Copy/paste parent/child -> pasted_parent/pasted_child
    let (pasted_parent, pasted_child) = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            None,
            vec![(parent.id, GEOMETRY1), (child.id, GEOMETRY2)],
        )
        .await?;
        assert_eq!(pasted.len(), 2);
        (
            Connectable::new(test, pasted[0]),
            Connectable::new(test, pasted[1]),
        )
    };

    // Set the pasted components' values to make sure those are flowing as expected
    pasted_parent.set_value(ctx, "pasted parent").await?;
    pasted_child.set_value(ctx, "pasted child").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure child infers its value from parent now
    assert_eq!(
        json!({
            "Value": "pasted child",
            "Inferred": "pasted parent"
        }),
        pasted_child.domain(ctx).await?
    );

    // Make sure original didn't change
    assert_eq!(
        json!({
            "Value": "child",
            "Inferred": "parent",
        }),
        child.domain(ctx).await?
    );

    Ok(())
}

#[test]
async fn paste_child_and_parent_opposite_order(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // parent and child
    let parent = test.create_parent(ctx, "parent").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let child = test.create_connectable(ctx, "child", None, []).await?;
    Frame::upsert_parent(ctx, child.id, parent.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        json!({
            "Value": "child",
            "Inferred": "parent",
        }),
        child.domain(ctx).await?
    );

    // Copy/paste child/parent -> pasted_child, pasted_parent
    let (pasted_child, pasted_parent) = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            None,
            vec![(child.id, GEOMETRY1), (parent.id, GEOMETRY2)],
        )
        .await?;
        assert_eq!(pasted.len(), 2);
        (
            Connectable::new(test, pasted[0]),
            Connectable::new(test, pasted[1]),
        )
    };

    // Set the pasted components' values to make sure those are flowing as expected
    pasted_parent.set_value(ctx, "pasted parent").await?;
    pasted_child.set_value(ctx, "pasted child").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure child infers its value from parent now
    assert_eq!(
        json!({
            "Value": "pasted child",
            "Inferred": "pasted parent"
        }),
        pasted_child.domain(ctx).await?
    );

    // Make sure original didn't change
    assert_eq!(
        json!({
            "Value": "child",
            "Inferred": "parent",
        }),
        child.domain(ctx).await?
    );

    Ok(())
}

#[test]
async fn paste_child_only(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // parent and child
    let parent = test.create_parent(ctx, "parent").await?;
    let child = test.create_connectable(ctx, "child", None, []).await?;
    Frame::upsert_parent(ctx, child.id, parent.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        json!({
            "Value": "child",
            "Inferred": "parent",
        }),
        child.domain(ctx).await?
    );

    // Copy/paste child -> pasted_child
    let pasted_child = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            None,
            vec![(child.id, GEOMETRY1)],
        )
        .await?;
        assert_eq!(pasted.len(), 1);
        Connectable::new(test, pasted[0])
    };

    // Set the pasted components' values to make sure those are flowing as expected
    pasted_child.set_value(ctx, "pasted child").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure child no longer gets a parent value
    assert_eq!(
        json!({
            "Value": "pasted child"
        }),
        pasted_child.domain(ctx).await?
    );

    // Make sure originals didn't change
    assert_eq!(
        json!({
            "Value": "child",
            "Inferred": "parent",
        }),
        child.domain(ctx).await?
    );

    Ok(())
}

#[test]
async fn paste_child_into_new_parent(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // parent and child
    let parent = test.create_parent(ctx, "parent").await?;
    let parent2 = test.create_parent(ctx, "parent2").await?;
    let child = test.create_connectable(ctx, "child", None, []).await?;
    Frame::upsert_parent(ctx, child.id, parent.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        json!({
            "Value": "child",
            "Inferred": "parent",
        }),
        child.domain(ctx).await?
    );

    // Copy/paste child -> pasted_child
    let pasted_child = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            Some(parent2.id),
            vec![(child.id, GEOMETRY1)],
        )
        .await?;
        assert_eq!(pasted.len(), 1);
        Connectable::new(test, pasted[0])
    };

    // Set the pasted components' values to make sure those are flowing as expected
    pasted_child.set_value(ctx, "pasted child").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Make sure child infers its value from parent now
    assert_eq!(
        json!({
            "Value": "pasted child",
            "Inferred": "parent2"
        }),
        pasted_child.domain(ctx).await?
    );

    Ok(())
}

#[test]
async fn paste_manager_and_managed(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;

    // manager and original
    let manager = test.create_manager(ctx, "manager").await?;
    let original = test.create_connectable(ctx, "original", None, []).await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Copy/paste manager/original -> pasted_manager/pasted
    let (pasted_manager, pasted) = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            None,
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
    let original = test.create_connectable(ctx, "original", None, []).await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Copy/paste original/manager -> pasted/pasted_manager
    let (pasted, pasted_manager) = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            None,
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
    let original = test.create_connectable(ctx, "original", None, []).await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Copy/paste manager -> pasted_manager
    let pasted_manager = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            None,
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
    let original = test.create_connectable(ctx, "original", None, []).await?;
    Component::manage_component(ctx, manager.id, original.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Copy/paste original -> pasted
    let pasted = {
        let pasted = Component::batch_copy(
            ctx,
            View::get_id_for_default(ctx).await?,
            None,
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

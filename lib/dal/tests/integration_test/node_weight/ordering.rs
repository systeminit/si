use dal::DalContext;
use dal_test::expected::{self, ExpectComponent};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

//
// Expecteds
//
// | Graph               | Updates             |
// |---------------------|---------------------|
// |                     | removed ordering    | node is removed
// | removed ordering    | removed ordering    | node is removed
// |                     | added   ordering    | node is added
// | add+remove ordinals | add/remove ordinals | new edge should

#[test]
async fn correct_transforms_no_corrections(ctx: &mut DalContext) {
    //
    // Make a docker image with two ExposedPorts
    //
    let component =
        expected::create_component_for_default_schema_name(ctx, "Docker Image", "a tulip in a cup")
            .await;
    let exposed_ports =
        ExpectComponent::prop(ctx, component.id(), "root/domain/ExposedPorts").await;
    exposed_ports.push(ctx, "1").await;
    exposed_ports.push(ctx, "2").await;
    assert_eq!(json!(["1", "2"]), exposed_ports.get(ctx).await);
    expected::apply_change_set_to_base(ctx).await;

    // Add 3, 4 and apply
    expected::fork_from_head_change_set(ctx).await;
    exposed_ports.push(ctx, "3").await;
    exposed_ports.push(ctx, "4").await;
    expected::apply_change_set_to_base(ctx).await;

    // Remove all edges and apply
    expected::fork_from_head_change_set(ctx).await;
    exposed_ports.update(ctx, None).await;
    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(exposed_ports.view(ctx).await, None);
}

#[test]
async fn correct_transforms_added_edges(ctx: &mut DalContext) {
    // Make a docker image with ExposedPorts = 1, 22, and 33
    let component =
        expected::create_component_for_default_schema_name(ctx, "Docker Image", "a tulip in a cup")
            .await;
    let exposed_ports =
        ExpectComponent::prop(ctx, component.id(), ["root", "domain", "ExposedPorts"]).await;
    exposed_ports.push(ctx, "1").await;
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(json!(["1"]), exposed_ports.get(ctx).await);

    // Fork a change set, remove 22 and add 2
    let change_set_2 = expected::fork_from_head_change_set(ctx).await;
    exposed_ports.push(ctx, "2").await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(json!(["1", "2"]), exposed_ports.get(ctx).await);

    // Fork a separate change set, remove 33 and add 3
    let change_set_3 = expected::fork_from_head_change_set(ctx).await;
    exposed_ports.push(ctx, "3").await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(json!(["1", "3"]), exposed_ports.get(ctx).await);

    // Apply both changesets
    expected::update_visibility_and_snapshot_to_visibility(ctx, change_set_2.id).await;
    assert_eq!(json!(["1", "2"]), exposed_ports.get(ctx).await);
    expected::apply_change_set_to_base(ctx).await;

    expected::update_visibility_and_snapshot_to_visibility(ctx, change_set_3.id).await;
    assert_eq!(json!(["1", "2", "3"]), exposed_ports.get(ctx).await);
}

#[test]
async fn correct_transforms_removed_edges(ctx: &mut DalContext) {
    // Make a docker image with ExposedPorts = 1, 22, and 33
    let component =
        expected::create_component_for_default_schema_name(ctx, "Docker Image", "a tulip in a cup")
            .await;
    let exposed_ports =
        ExpectComponent::prop(ctx, component.id(), ["root", "domain", "ExposedPorts"]).await;
    exposed_ports.push(ctx, "1").await;
    exposed_ports.push(ctx, "33").await;
    exposed_ports.push(ctx, "22").await;
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(json!(["1", "33", "22"]), exposed_ports.get(ctx).await);

    // Fork a change set, remove 22 and add 2
    let change_set_2 = expected::fork_from_head_change_set(ctx).await;
    exposed_ports.remove_child_at(ctx, 2).await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(json!(["1", "33"]), exposed_ports.get(ctx).await);

    // Fork a separate change set, remove 33 and add 3
    let change_set_3 = expected::fork_from_head_change_set(ctx).await;
    exposed_ports.remove_child_at(ctx, 1).await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(json!(["1", "22"]), dbg!(exposed_ports.get(ctx).await));

    // Apply both changesets
    expected::update_visibility_and_snapshot_to_visibility(ctx, change_set_2.id).await;
    assert_eq!(json!(["1", "33"]), exposed_ports.get(ctx).await);
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(json!(["1", "33"]), exposed_ports.get(ctx).await);

    expected::update_visibility_and_snapshot_to_visibility(ctx, change_set_3.id).await;
    // 22 was removed in change set 2, which was applied, so we're down to one
    assert_eq!(json!(["1"]), exposed_ports.get(ctx).await);
    expected::apply_change_set_to_base(ctx).await;

    // 33 was removed in change set 3, so we end up with just "1"
    assert_eq!(json!(["1"]), exposed_ports.get(ctx).await);
}

#[test]
async fn correct_transforms_both_added_and_removed_edges(ctx: &mut DalContext) {
    // Make a docker image with ExposedPorts = 1, 22, and 33
    let component =
        expected::create_component_for_default_schema_name(ctx, "Docker Image", "a tulip in a cup")
            .await;
    let exposed_ports =
        ExpectComponent::prop(ctx, component.id(), ["root", "domain", "ExposedPorts"]).await;
    exposed_ports.push(ctx, "1").await;
    exposed_ports.push(ctx, "33").await;
    exposed_ports.push(ctx, "22").await;
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(json!(["1", "33", "22"]), exposed_ports.get(ctx).await);

    // Fork a change set, remove 22 and add 2
    let change_set_2 = expected::fork_from_head_change_set(ctx).await;
    // Add "2" and remove "22"
    exposed_ports.push(ctx, "2").await;
    exposed_ports.remove_child_at(ctx, 2).await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(json!(["1", "33", "2"]), exposed_ports.get(ctx).await);

    // Fork a separate change set, remove 33 and add 3
    let change_set_3 = expected::fork_from_head_change_set(ctx).await;
    // Add "3" and remove "33"
    exposed_ports.push(ctx, "3").await;
    exposed_ports.remove_child_at(ctx, 1).await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(json!(["1", "22", "3"]), exposed_ports.get(ctx).await);

    // Apply both changesets
    expected::update_visibility_and_snapshot_to_visibility(ctx, change_set_2.id).await;
    assert_eq!(json!(["1", "33", "2"]), exposed_ports.get(ctx).await);
    expected::apply_change_set_to_base(ctx).await;

    expected::update_visibility_and_snapshot_to_visibility(ctx, change_set_3.id).await;
    // The removal of "22" from the applied change set above will be reflected
    // in the change set, and the addition of "2" will come in
    assert_eq!(json!(["1", "2", "3"]), exposed_ports.get(ctx).await);
    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(json!(["1", "2", "3"]), exposed_ports.get(ctx).await);
}

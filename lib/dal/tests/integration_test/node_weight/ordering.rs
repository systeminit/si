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
    let component = ExpectComponent::create(ctx, "Docker Image").await;
    let exposed_ports = component
        .prop(ctx, ["root", "domain", "ExposedPorts"])
        .await;
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
    let component = ExpectComponent::create(ctx, "Docker Image").await;
    let exposed_ports = component
        .prop(ctx, ["root", "domain", "ExposedPorts"])
        .await;
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
    let component = ExpectComponent::create(ctx, "Docker Image").await;
    let exposed_ports = component
        .prop(ctx, ["root", "domain", "ExposedPorts"])
        .await;
    exposed_ports.push(ctx, "1").await;
    exposed_ports.push(ctx, "33").await;
    exposed_ports.push(ctx, "22").await;
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(json!(["1", "33", "22"]), exposed_ports.get(ctx).await);

    // Fork a change set, remove 22 and add 2
    let change_set_2 = expected::fork_from_head_change_set(ctx).await;
    exposed_ports.children(ctx).await[2].remove(ctx).await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(json!(["1", "33"]), exposed_ports.get(ctx).await);

    // Fork a separate change set, remove 33 and add 3
    let change_set_3 = expected::fork_from_head_change_set(ctx).await;
    exposed_ports.children(ctx).await[1].remove(ctx).await;
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
    let component = ExpectComponent::create(ctx, "Docker Image").await;
    let exposed_ports = component
        .prop(ctx, ["root", "domain", "ExposedPorts"])
        .await;
    exposed_ports.push(ctx, "1").await;
    exposed_ports.push(ctx, "33").await;
    exposed_ports.push(ctx, "22").await;
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(json!(["1", "33", "22"]), exposed_ports.get(ctx).await);

    // Fork a change set, remove 22 and add 2
    let change_set_2 = expected::fork_from_head_change_set(ctx).await;
    // Add "2" and remove "22"
    exposed_ports.push(ctx, "2").await;
    exposed_ports.children(ctx).await[2].remove(ctx).await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(json!(["1", "33", "2"]), exposed_ports.get(ctx).await);

    // Fork a separate change set, remove 33 and add 3
    let change_set_3 = expected::fork_from_head_change_set(ctx).await;
    // Add "3" and remove "33"
    exposed_ports.push(ctx, "3").await;
    exposed_ports.children(ctx).await[1].remove(ctx).await;
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

#[test]
async fn correct_transforms_attribute_value_duplicate_map_keys(ctx: &mut DalContext) {
    // Make a docker image with ExposedPorts = 1, 22, and 33
    let component = ExpectComponent::create(ctx, "pirate").await;
    let treasure = component.prop(ctx, ["root", "domain", "treasure"]).await;
    treasure.push_with_key(ctx, "a", "1").await;
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(json!({"a":  "1"}), treasure.get(ctx).await);

    // Fork a change set, add "b", "c", "d" and "e"
    let change_set_2 = expected::fork_from_head_change_set(ctx).await;
    treasure.push_with_key(ctx, "b", "2").await;
    treasure.push_with_key(ctx, "c", "3").await;
    treasure.push_with_key(ctx, "d", "4").await;
    treasure.push_with_key(ctx, "e", "5").await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(
        json!({"a": "1", "b": "2", "c": "3", "d": "4", "e": "5"}),
        treasure.get(ctx).await
    );

    // Fork a separate change set, add duplicates c and d, non duplicate f
    let _change_set_3 = expected::fork_from_head_change_set(ctx).await;
    treasure.push_with_key(ctx, "c", "300").await;
    treasure.push_with_key(ctx, "d", "400").await;
    treasure.push_with_key(ctx, "f", "6").await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(
        json!({"a": "1", "c": "300", "d": "400", "f": "6"}),
        treasure.get(ctx).await
    );
    // Apply change_set_3
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(
        json!({"a": "1", "c": "300", "d": "400", "f": "6"}),
        treasure.get(ctx).await
    );

    // Update to change set 2, and check that we got the new key, and replaced the duplicates
    expected::update_visibility_and_snapshot_to_visibility(ctx, change_set_2.id).await;
    assert_eq!(
        json!({"a": "1", "b": "2", "c": "300", "d": "400", "e": "5", "f": "6"}),
        treasure.get(ctx).await
    );
    assert_eq!(
        6,
        treasure
            .attribute_value(ctx)
            .await
            .children(ctx)
            .await
            .len()
    );
}
